//! RINEX decompression module
use crate::{
    hatanaka::{Error, NumDiff, ObsDiff, TextDiff, CRINEX},
    prelude::{Version, SV},
};

use std::{
    collections::HashMap,
    io::{Read, Result},
    str::FromStr,
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum State {
    /// Inside File Header
    #[default]
    Header,
    /// Inside an [Epoch] descriptor
    EpochDescriptor,
}

/// Structure to decompress CRINEX data
pub struct Decompressor<const M: usize, R: Read> {
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
    /// [R]
    reader: R,
    /// Internal buffer
    buf: [u8; 1024],
    /// Partial line buffer
    /// is used to cover the possibility of frame recovered by 2 successive read
    partial_line: [u8; 128],
    /// Read pointer
    ptr: usize,
    /// counters
    nb_sv: usize, // total
    sv_ptr: usize, // inside epoch
    /// True only for first epoch ever processed
    first_epoch: bool,
    /// True until one clock offset was found
    first_clock_diff: bool,
    /// Epoch differentiator
    epoch_diff: TextDiff,
    /// Current readable buffer
    buf_ascii: String,
    /// Unformatted Readable Internal buffer.
    epoch_ascii: String,
    /// Clock offset differentiator
    clock_diff: NumDiff<M>,
    /// Observation differentiators
    obs_diff: HashMap<SV, ObsDiff<M>>,
    /// CRINEX header that we did identify.
    /// You should wait until State != State::InsideHeader to consider this field.
    pub crinex: CRINEX,
}

impl<const M: usize, R: Read> Read for Decompressor<M, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.state {
            State::Header => self.read_header(buf),
            _ => Ok(0),
        }
    }
}

// /// Reworks given content to match RINEX specifications
// /// of an epoch descriptor
// fn format_epoch(
//     version: u8,
//     nb_sv: usize,
//     content: &str,
//     clock_offset: Option<i64>,
// ) -> Result<String, Error> {

//     let mut result = String::new();
//     match version {
//         1 | 2 => {
//             // old RINEX
//             // append Systems #ID,
//             //  on as many lines as needed
//             let min_size = 32 + 3; // epoch descriptor + at least one vehicle
//             if content.len() < min_size {
//                 // parsing would fail
//                 return Err(Error::FaultyRecoveredEpoch);
//             }

//             let (epoch, systems) = content.split_at(32); // grab epoch
//             result.push_str(&epoch.replace('&', " ")); // rework

//             //CRINEX has systems squashed in a single line
//             // we just split it to match standard definitions
//             // .. and don't forget the tab
//             if nb_sv <= 12 {
//                 // fits in a single line
//                 result.push_str(systems);
//                 if let Some(value) = clock_offset {
//                     result.push_str(&format!("  {:3.9}", (value as f64) / 1000.0_f64))
//                 }
//             } else {
//                 // does not fit in a single line
//                 let mut index = 0;
//                 for i in 0..nb_sv {
//                     if index == 12 {
//                         index = 0;
//                         if i == 12 {
//                             // first line,
//                             if let Some(value) = clock_offset {
//                                 result.push_str(&format!("  {:3.9}", (value as f64) / 1000.0_f64))
//                             }
//                         }
//                         // tab indent
//                         result.push_str("\n                                "); //TODO: improve this please
//                     }
//                     /*
//                      * avoids overflowing
//                      */
//                     let min_offset = i * 3;
//                     let max_offset = std::cmp::min(min_offset + 3, systems.len());
//                     result.push_str(&systems[min_offset..max_offset]);
//                     index += 1;
//                 }
//             }
//         },
//         _ => {
//             // Modern RINEX case
//             // Systems #ID to be passed on future lines
//             if content.len() < 35 {
//                 // parsing would fail
//                 return Err(Error::FaultyRecoveredEpoch);
//             }
//             let (epoch, _) = content.split_at(35);
//             result.push_str(&epoch.replace('&', " "));
//             //TODO clock offset
//             if let Some(value) = clock_offset {
//                 result.push_str(&format!("         {:3.12}", (value as f64) / 1000.0_f64))
//             }
//         },
//     }
//     Ok(result)
// }

impl<const M: usize, R: Read> Decompressor<M, R> {
    /// Creates a new [Decompressor] working from [Read]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            nb_sv: 0,
            sv_ptr: 0,
            ptr: 0,
            buf: [0; 1024],
            partial_line: [0; 128],
            first_epoch: true,
            first_clock_diff: true,
            state: State::default(),
            crinex: CRINEX::default(),
            epoch_diff: TextDiff::new(""),
            clock_diff: NumDiff::<M>::new(0),
            obs_diff: HashMap::new(), // init. late
            buf_ascii: String::with_capacity(128),
            epoch_ascii: String::with_capacity(256),
        }
    }

    /// When parsing (algorithm being active)
    /// any COMMENTS encountered should be passed "as is".
    /// There is no mean to apply the Hatanaka algorithm to COMMENTS located
    /// in the file body
    fn process_buffered_comments(&mut self, buf: &mut [u8]) -> usize {
        let mut offset = 0;
        let buf_len = self.buf.len();
        for chunk in 0..buf_len / 80 {
            // For all encountered COMMENTS simply extract as is
            let start = chunk * 80;
            let stop = ((chunk + 1) * 80).min(buf_len);
            let size = stop - start;
            // Using .ends_with is a nice way to
            //    - Only grab valid descriptor
            //    - Only grab complete lines
            //    Partial comments are to be treated next time
            if self.buf_ascii[start..stop].ends_with("COMMENTS") {
                buf[start..stop].copy_from_slice(&self.buf[start..stop]);
                offset += size;
            }
        }
        offset
    }

    /// In the header section, we need to search for the CRINEX header
    /// and we also have to serve the data "as is": we act like a FIFO.
    /// Once End of Header is specified, this state will be over
    /// and decompression scheme needs to deploy.
    fn read_header(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut read_size = buf.len();
        let mut next_state = self.state;

        if self.ptr > 0 {
            // we have some leftovers ready to be exposed
            if self.ptr >= read_size {
                // We have more than needed: expose and exit: no Read
                buf[..read_size].copy_from_slice(&self.buf[self.ptr..read_size]);
                self.ptr += read_size;
                return Ok(read_size);
            } else {
                // We don't have enough:
                //  1. Expose what we have
                //  2. new Read
                buf[..self.ptr].copy_from_slice(&self.buf[self.ptr..]);
                self.ptr = 0;
            }
        }

        // Fill internal buffer
        let new_size = self.reader.read(&mut self.buf)?;
        if new_size == 0 {
            // End of stream: notify and exit
            return Ok(0);
        }

        // study new (complete) lines, one at a time
        // all correctly studied content (= complete + valid line)
        // gets forwared to user
        let mut ptr = 0;
        while ptr < new_size && next_state == State::Header {
            let (start, stop) = (ptr, (ptr + 80).min(new_size));
            let ascii = String::from_utf8_lossy(&self.buf[start..stop]);
            println!("ASCII \"{}\"", ascii);

            if stop > read_size {
                // can't forward this
                break;
            }
            if ascii.len() < 80 {
                // not a complete line: not treated
                continue;
            }

            // forward as is (=header)
            buf[start..stop].copy_from_slice(&self.buf[start..stop]);

            if ascii.contains("CRINEX VERS") {
                let version = Version::from_str(&ascii[0..20].trim())
                    .map_err(|_| Error::VersionParsing.to_stdio())?;

                self.crinex.with_version(version);
            } else if ascii.ends_with("CRINEX PROG / DATE") {
                self.crinex.with_prog_date(&ascii[0..60].trim());
            } else if ascii.contains("END OF HEADER") {
                next_state = State::EpochDescriptor;
            }

            ptr += ascii.len() + 1;
        }

        // // forward all that we can
        // let read_size = ptr.min(read_size);
        // buf[..read_size].copy_from_slice(&self.buf[..read_size]);

        // determine next state
        match (self.state, next_state) {
            (State::Header, State::EpochDescriptor) => {},
            _ => {}, // stable or invalid combinations
        }

        println!("NEXT STATE: {:?}", next_state);
        self.state = next_state;
        Ok(ptr)
    }

    // fn parse_nb_sv(content: &str, crx_major: u8) -> Option<usize> {
    //     let mut offset: usize = 2    // Y
    //         +2+1 // m
    //         +2+1 // d
    //         +2+1 // h
    //         +2+1 // m
    //         +11  // s
    //         +1   // ">" or "&" init marker
    //         +3; // epoch flag

    //     if content.starts_with("> ") {
    //         //CRNX3 initial epoch, 1 extra whitespace
    //         offset += 1;
    //     }

    //     if crx_major > 1 {
    //         offset += 2; // YYYY on 4 digits
    //     }

    //     let (_, rem) = content.split_at(offset);
    //     let (n, _) = rem.split_at(3);
    //     if let Ok(n) = u16::from_str_radix(n.trim(), 10) {
    //         Some(n.into())
    //     } else {
    //         None
    //     }
    // }

    // fn parse_flags(&mut self, sv: SV, content: &str) {
    //     //println!("FLAGS: \"{}\"", content); // DEBUG
    //     if let Some(sv_diff) = self.sv_diff.get_mut(&sv) {
    //         for index in 0..content.len() {
    //             if let Some(sv_obs) = sv_diff.get_mut(index / 2) {
    //                 if index % 2 == 0 {
    //                     // LLI
    //                     let _ = sv_obs.1.decompress(&content[index..index + 1]);
    //                 } else {
    //                     //SSI
    //                     let _ = sv_obs.2.decompress(&content[index..index + 1]);
    //                 }
    //             }
    //         }
    //     }
    // }

    // fn current_satellite(
    //     &self,
    //     crx_major: u8,
    //     crx_constellation: &Constellation,
    //     sv_ptr: usize,
    // ) -> Option<SV> {
    //     let epoch = &self.epoch_descriptor;
    //     let offset: usize = match crx_major {
    //         1 => std::cmp::min(32 + 3 * (sv_ptr + 1), epoch.len()), // overflow protection
    //         _ => std::cmp::min(41 + 3 * (sv_ptr + 1), epoch.len()), // overflow protection
    //     };
    //     let system = epoch.split_at(offset).0;
    //     let (_, svnn) = system.split_at(system.len() - 3); // last 3 XXX
    //     let svnn = svnn.trim();
    //     match crx_major > 2 {
    //         false => {
    //             // OLD
    //             match crx_constellation {
    //                 Constellation::Mixed => {
    //                     // OLD and MIXED is fine
    //                     if let Ok(sv) = SV::from_str(svnn) {
    //                         Some(sv)
    //                     } else {
    //                         None
    //                     }
    //                 },
    //                 constellation => {
    //                     // OLD + FIXED: constellation might be omitted.......
    //                     if let Ok(prn) = u8::from_str_radix(svnn[1..].trim(), 10) {
    //                         Some(SV {
    //                             prn,
    //                             constellation: *constellation,
    //                         })
    //                     } else {
    //                         None
    //                     }
    //                 },
    //             }
    //         },
    //         true => {
    //             // MODERN
    //             if let Ok(sv) = SV::from_str(svnn) {
    //                 Some(sv)
    //             } else {
    //                 None
    //             }
    //         },
    //     }
    // }

    // /// Decompresses (recovers) RINEX from given CRINEX content.
    // /// This method expects either RINEX comments,
    // /// or CRNX1/CRNX3 content, that is either epoch description
    // /// and epoch content.
    // pub fn decompress(
    //     &mut self,
    //     crx_major: u8,
    //     crx_constell: &Constellation,
    //     rnx_major: u8,
    //     observables: &HashMap<Constellation, Vec<Observable>>,
    //     content: &str,
    // ) -> Result<String, Error> {
    //     // content browser
    //     let mut result: String = String::new();
    //     let mut lines = content.lines();
    //     loop {
    //         // browse all provided lines
    //         let line: &str = match lines.next() {
    //             Some(l) => l,
    //             None => break,
    //         };

    //         //println!("DECOMPRESSING - \"{}\"", line); //DEBUG
    //         //println!("state: {:?}", self.state);

    //         // [0] : COMMENTS (special case)
    //         if is_rinex_comment(line) {
    //             //if line.contains("RINEX FILE SPLICE") {
    //             // [0*] SPLICE special comments
    //             //      merged RINEX Files
    //             //    self.reset();
    //             //}
    //             result // feed content as is
    //                 .push_str(line);
    //             result.push('\n');
    //             continue; // move to next line
    //         }

    //         // [0*]: special epoch events
    //         //       with uncompressed descriptor
    //         //       (CRNX3)
    //         if line.starts_with("> ") && !self.first_epoch {
    //             result // feed content as is
    //                 .push_str(line);
    //             result.push('\n');
    //             continue; // move to next line
    //         }

    //         match self.state {
    //             State::EpochDescriptor => {
    //                 if self.first_epoch {
    //                     match crx_major {
    //                         1 => {
    //                             if !line.starts_with('&') {
    //                                 return Err(Error::FaultyCrx1FirstEpoch);
    //                             }
    //                         },
    //                         3 => {
    //                             if !line.starts_with('>') {
    //                                 return Err(Error::FaultyCrx3FirstEpoch);
    //                             }
    //                         },
    //                         _ => {}, // will never happen
    //                     }

    //                     // Kernel initialization,
    //                     // only once, always text based
    //                     // from this entire line
    //                     self.epoch_diff.init(line.trim_end());
    //                     self.first_epoch = false;
    //                 } else {
    //                     /*
    //                      * this latches the current line content
    //                      * we'll deal with it when combining with clock offsets
    //                      */
    //                     self.epoch_diff.decompress(line);
    //                 }

    //                 self.state = State::ClockOffsetDescriptor;
    //             }, // state::EpochDescriptor

    //             State::ClockOffsetDescriptor => {
    //                 /*
    //                  * this line is dedicated to clock offset description
    //                  */
    //                 let mut clock_offset: Option<i64> = None;
    //                 if line.contains('&') {
    //                     // clock offset kernel (re)init
    //                     let (n, rem) = line.split_at(1);
    //                     if let Ok(order) = u8::from_str_radix(n, 10) {
    //                         let (_, value) = rem.split_at(1);

    //                         if let Ok(value) = i64::from_str_radix(value, 10) {
    //                             self.clock_diff.force_init(value);
    //                         } else {
    //                             return Err(Error::ClockOffsetValueError);
    //                         }
    //                     } else {
    //                         return Err(Error::ClockOffsetOrderError);
    //                     }
    //                 } else {
    //                     // --> nominal clock offset line
    //                     if let Ok(value) = i64::from_str_radix(line.trim(), 10) {
    //                         clock_offset = Some(value); // latch for later
    //                     }
    //                 }

    //                 /*
    //                  * now we have all information to reconstruct the epoch descriptor
    //                  */
    //                 let recovered = self.epoch_diff.decompress(" ").trim_end();
    //                 // we store the recovered and unformatted CRINEX descriptor
    //                 //   because it is particularly easy to parse,
    //                 //   as it is made of a single line.
    //                 //   It needs to be formatted according to standards,
    //                 //   for the result being constructed. See the following operations
    //                 self.epoch_descriptor = recovered.to_string();
    //                 // initialize sv identifier
    //                 self.sv_ptr = 0;
    //                 if let Some(n) = Self::parse_nb_sv(&self.epoch_descriptor, crx_major) {
    //                     self.nb_sv = n;
    //                 } else {
    //                     return Err(Error::VehicleIdentificationError);
    //                 }

    //                 if let Ok(descriptor) =
    //                     format_epoch(rnx_major, self.nb_sv, recovered, clock_offset)
    //                 {
    //                     //println!("--- EPOCH --- \n{}[STOP]", descriptor.trim_end()); //DEBUG
    //                     result.push_str(&format!("{}\n", descriptor.trim_end()));
    //                 } else {
    //                     return Err(Error::EpochConstruct);
    //                 }

    //                 self.state = State::Body;
    //             }, // state::ClockOffsetDescriptor

    //             State::Body => {
    //                 let mut obs_ptr: usize = 0;
    //                 let mut observations: Vec<Option<i64>> = Vec::with_capacity(16);
    //                 /*
    //                  * identify satellite we're dealing with
    //                  */
    //                 if let Some(sv) = self.current_satellite(crx_major, crx_constell, self.sv_ptr) {
    //                     //println!("SV: {:?}", sv); //DEBUG
    //                     self.sv_ptr += 1; // increment for next time
    //                                       // vehicles are always described in a single line
    //                     if rnx_major > 2 {
    //                         // RNX3 needs SVNN on every line
    //                         result.push_str(&format!("{} ", sv));
    //                     }
    //                     /*
    //                      * Build compress tools in case this vehicle is new
    //                      */
    //                     if self.sv_diff.get(&sv).is_none() {
    //                         let mut inner: Vec<(NumDiff<3>, TextDiff, TextDiff)> =
    //                             Vec::with_capacity(16);
    //                         // this protects from malformed Headers or malformed Epoch descriptions
    //                         let codes = match sv.constellation.is_sbas() {
    //                             true => observables.get(&Constellation::SBAS),
    //                             false => observables.get(&sv.constellation),
    //                         };
    //                         if let Some(codes) = codes {
    //                             for _ in codes {
    //                                 let mut kernels =
    //                                     (NumDiff::<3>::new(0), TextDiff::new(), TextDiff::new());

    //                                 kernels.1.init(" "); // LLI
    //                                 kernels.2.init(" "); // SSI
    //                                 inner.push(kernels);
    //                             }
    //                             self.sv_diff.insert(sv, inner);
    //                         }
    //                     }
    //                     /*
    //                      * iterate over entire line
    //                      */
    //                     let mut line = line.trim_end();
    //                     let codes = match sv.constellation.is_sbas() {
    //                         true => observables.get(&Constellation::SBAS),
    //                         false => observables.get(&sv.constellation),
    //                     };
    //                     if let Some(codes) = codes {
    //                         while obs_ptr < codes.len() {
    //                             if let Some(pos) = line.find(' ') {
    //                                 let content = &line[..pos];
    //                                 //println!("OBS \"{}\" - CONTENT \"{}\"", codes[obs_ptr], content); //DEBUG
    //                                 if content.is_empty() {
    //                                     /*
    //                                      * missing observation
    //                                      */
    //                                     observations.push(None);
    //                                 } else {
    //                                     /*
    //                                      * regular progression
    //                                      */
    //                                     if let Some(sv_diff) = self.sv_diff.get_mut(&sv) {
    //                                         if let Some(marker) = content.find('&') {
    //                                             // kernel (re)initialization
    //                                             let (order, rem) = content.split_at(marker);
    //                                             let order = u8::from_str_radix(order.trim(), 10)?;
    //                                             //println!("ORDER {}", order); //DEBUG
    //                                             let (_, data) = rem.split_at(1);
    //                                             if let Ok(data) =
    //                                                 i64::from_str_radix(data.trim(), 10)
    //                                             {
    //                                                 sv_diff[obs_ptr]
    //                                                     .0 // observations only, at this point
    //                                                     .force_init(data);

    //                                                 observations.push(Some(data));
    //                                             }
    //                                         } else {
    //                                             // regular compression
    //                                             if let Ok(num) =
    //                                                 i64::from_str_radix(content.trim(), 10)
    //                                             {
    //                                                 let recovered = sv_diff[obs_ptr]
    //                                                     .0 // observations only, at this point
    //                                                     .decompress(num);
    //                                                 observations.push(Some(recovered));
    //                                             }
    //                                         }
    //                                     }
    //                                 }
    //                                 line = &line[std::cmp::min(pos + 1, line.len())..]; // line remainder
    //                                 obs_ptr += 1;
    //                             } else {
    //                                 /*
    //                                  * EOL detected, but obs_ptr < codes.len()
    //                                  *  => try to parse one last obs
    //                                  */
    //                                 //println!("OBS \"{}\" - CONTENT \"{}\"", codes[obs_ptr], line); //DEBUG
    //                                 if let Some(sv_diff) = self.sv_diff.get_mut(&sv) {
    //                                     if let Some(marker) = line.find('&') {
    //                                         // kernel (re)initliaization
    //                                         let (order, rem) = line.split_at(marker);
    //                                         let order = u8::from_str_radix(order.trim(), 10)?;
    //                                         let (_, data) = rem.split_at(1);
    //                                         if let Ok(data) = i64::from_str_radix(data.trim(), 10) {
    //                                             sv_diff[obs_ptr]
    //                                                 .0 // observations only, at this point
    //                                                 .force_init(data);

    //                                             observations.push(Some(data));
    //                                         }
    //                                     } else {
    //                                         // regular compression
    //                                         if let Ok(num) = i64::from_str_radix(line.trim(), 10) {
    //                                             let recovered = sv_diff[obs_ptr]
    //                                                 .0 // observations only, at this point
    //                                                 .decompress(num);
    //                                             observations.push(Some(recovered))
    //                                         }
    //                                     }
    //                                 } //svdiff
    //                                 line = ""; // avoid flags parsing: all flags omitted <=> content unchanged
    //                                 obs_ptr = codes.len();
    //                             } //EOL
    //                         } //while()
    //                     } //observables identification
    //                       /*
    //                        * Flags field
    //                        */
    //                     if !line.is_empty() {
    //                         // can parse at least 1 flag
    //                         self.parse_flags(sv, line);
    //                     }
    //                     /*
    //                      * group previously parsed observations,
    //                      *   into a single formatted line
    //                      *   or into several in case of OLD RINEX
    //                      */
    //                     for (index, data) in observations.iter().enumerate() {
    //                         if let Some(data) = data {
    //                             let sv_diff = self.sv_diff.get_mut(&sv).unwrap(); //cant fail at this point
    //                             let lli = sv_diff[index]
    //                                 .1 // LLI
    //                                 .decompress(" ") // trick to recover
    //                                 // using textdiff property.
    //                                 // Another option would be to have an array to
    //                                 // store them
    //                                 .to_string();
    //                             let ssi = sv_diff[index]
    //                                 .2 // SSI
    //                                 .decompress(" ") // trick to recover
    //                                 // using textdiff property.
    //                                 // Another option would be to have an array to
    //                                 // store them
    //                                 .to_string();
    //                             result.push_str(&format!(
    //                                 "{:13.3}{}{} ",
    //                                 *data as f64 / 1000_f64,
    //                                 lli,
    //                                 ssi
    //                             )); //F14.3
    //                         } else {
    //                             result.push_str("                "); // BLANK
    //                         }

    //                         if rnx_major < 3 {
    //                             // old RINEX
    //                             if (index + 1).rem_euclid(5) == 0 {
    //                                 // maximal nb of OBS per line
    //                                 result.push('\n')
    //                             }
    //                         }
    //                     }
    //                     //result.push_str("\n");
    //                 }
    //                 // end of line parsing
    //                 //  if sv_ptr has reached the expected amount of vehicles
    //                 //  we reset to state (1)
    //                 if self.sv_ptr >= self.nb_sv {
    //                     self.state = State::EpochDescriptor;
    //                 }
    //             }, //current_satellite()
    //         } //match(state)
    //     } //loop
    //       //println!("--- TOTAL DECOMPRESSED --- \n\"{}\"", result); //DEBUG
    //     Ok(result)
    // }
}
