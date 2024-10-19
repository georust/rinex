//! CRINEX decompression module
use crate::{
    hatanaka::{Error, NumDiff, ObsDiff, TextDiff, CRINEX},
    prelude::{Version, SV},
};

use std::{
    collections::HashMap,
    io::{Read, Result},
    str::{from_utf8, FromStr},
};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum State {
    /// Inside File Header
    #[default]
    Header,
    /// Decypher Epoch timestamp
    Timestamp,
    /// Decypher Numsat
    NumSat,
    /// Decypher Satellites
    Satellites,
    /// Decypher Clock
    Clock,
    /// Observation
    Observation,
    /// LLI
    LLI,
    /// SNR
    SNR,
    /// Separator
    Separator,
    /// Content is discarded until a new Epoch starts.
    /// This will create a data gap. But we are resilient to boggus part
    /// of the data stream. This currently only happens when the Numsat description is boggus.
    // TODO: This could be improved. Recovered Epoch contains the numsat
    // information by itself, and we already handle boggus content inside of it.
    // It would slightly increase our flexibility with respect to invalid encoded content.
    Garbage,
}

impl State {
    /// Returns true if we're inside the File Body.
    /// Use this to grab the [CRINEX] definition if you have to.
    pub fn file_body(&self) -> bool {
        *self != Self::Header
    }
    /// Total bytes buffer should comprise so we can proceed and attempt
    /// moving to next [State]. We keep reading until buffer
    /// has enough data, otherwise it is pointless to attempt
    /// progressing in the FSM.
    pub(crate) fn expected_size(&self, v3: bool, numsat: usize) -> usize {
        match *self {
            Self::Header => 80,
            Self::Timestamp => {
                if v3 {
                    10
                } else {
                    12 // Y2000
                }
            },
            Self::NumSat => 3,
            Self::Satellites => 3 * numsat,
            Self::Clock => 17,
            Self::Observation => 17,
            Self::LLI | Self::SNR | Self::Separator => 1,
            Self::Garbage => 0, // not applicable
        }
    }
}

/// Structure to decompress CRINEX data
// Side notes (dev/expert)
//  Decompressor discards trailing whitespace (between last ASCII byte and \n)
//  - Which should most likely not exist in the Header section anyway
//  - Which might be present in the Epoch encoding process (complex process).
//  This kind of "purifies" the forwarded data, and actually facilitates the
//  parsing process coming right after.
pub struct Decompressor<const M: usize, R: Read> {
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
    /// For internal needs
    prev_state: State,
    /// [R]
    reader: R,
    /// Internal buffer
    buf: [u8; 1024],
    /// Pointers
    ptr: usize,
    wr_ptr: usize,
    /// counters
    numsat: usize, // total
    sv_ptr: usize, // inside epoch
    /// True only for first epoch ever processed
    first_epoch: bool,
    /// True until one clock offset was found
    first_clock_diff: bool,
    /// [TextDiff] that works on entire Epoch line
    epoch_diff: TextDiff,
    /// Current recovered epoch description
    epoch_descriptor: String,
    /// Current readable buffer
    buf_ascii: String,
    /// Unformatted Readable Internal buffer.
    epoch_ascii: String,
    /// Clock offset differentiator
    clock_diff: NumDiff<M>,
    /// Observation differentiators
    obs_diff: HashMap<SV, ObsDiff<M>>,
    /// Whether the CRINEX header was found or not
    crinex_found: bool,
    /// CRINEX header that we did identify.
    /// You should wait until State != State::InsideHeader to consider this field.
    pub crinex: CRINEX,
}

impl<const M: usize, R: Read> Read for Decompressor<M, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        println!("state: {:?}", self.state); //TODO: debug!

        // // COMMENTS cannot go through the decompression algorithm
        // // and must always be forwarded as is.
        // // Current behavior is to pass COMMENT groupings entirely
        // let wr_size = self.process_buffered_comments(buf);
        // if wr_size > 0 {
        //     return Ok(wr_size);
        // }

        if self.state == State::Header {
            // special initial case is treated here
            return self.read_header(buf);
        }

        // other states
        let buf_len = buf.len();
        let len = self.buf.len();

        if len < 80 {
            // we don't proceed. This takes care of
            // partial COMMENTS that could potentialy enter the decompression algorithm
            let size = self.reader.read(&mut self.buf)?;
            if size == 0 {
                // EOS reached
                return Ok(0);
            } else {
                return Err(Error::NeedMoreData.to_stdio());
            }
        }

        Ok(0)

        // let mut rd_size = 0;

        // // Process all data, running the FSM
        // // and forwarding data to user when we can
        // while self.ptr < len {

        //     let mut next_state = self.state;
        //     let avail = len - self.ptr;
        //     let required_size = self.state.expected_size();

        //     if avail < required_size {
        //         // cant proceed for this state
        //         if self.state != State::Clock {
        //             // it is possible that Clock is not produced
        //             // therefore we need to handle its presence
        //             if rd_size > 0 {
        //                 return Ok(rd_size);
        //             } else {
        //                 return Err(Error::NeedMoreData.to_stdio());
        //             }
        //         }
        //     }

        //     // For any state, we don't proceed if User can't accept the potential results.
        //     // It reduces the I/O performance, but simplifies internal logic.
        //     // Otherwise, we would need two separate FSM
        //     //   1. one for decypher process (with its own buffer)
        //     //   2. one for what was forwarded by the decyphering process (with its own buffer)

        //     // FSM
        //     match self.state {
        //         State::Timestamp => {
        //             next_state = State::NumSat;
        //         },
        //         State::NumSat => {
        //             next_state = State::Satellites;
        //         },
        //         State::Satellites => {
        //             next_state = State::Clock;
        //         },
        //         State::Clock => {
        //             // TODO: handle clock presence

        //             // Reached end of line
        //             //  - We proceed to run the textdiff and clock.numdiff
        //             //  - Numsat must be valid integer otherwise
        //             //  we can't progress and following decypher would panic (hard to debug).
        //             //  It is possible that the sat descriptor is invalid (on boggus encoder),
        //             //  but that we handle in the decyphering process individually.
        //             //  If NumSat is faulty (too boggus): we move on to the special
        //             //  [State::Garbage] state.
        //             let epoch_ascii = from_utf8(&self.buf[self.ptr..self.ptr+required_size])?;
        //             println!("epoch ascii: \"{}\"", epoch_ascii); // TODO: debug!

        //             // Forward input as is to the kernel (it is very important
        //             // otherwise we would bias the results)
        //             self.epoch_descriptor = self.epoch_diff
        //                 .decompress(&ascii);

        //             // It facilitates everything if we keep clean internal content
        //             self.epoch_descriptor = self.epoch_descriptor.trim();

        //             // Recover the clock offset

        //             // parse actual numsat
        //             // we don't need more th
        //             println!("descriptor: \"{}\"", self.epoch_descriptor); // TODO: debug!

        //             next_state = State::Observation;
        //         },
        //         State::Observation => {

        //         },
        //         State::LLI => {

        //         },
        //         State::SNR => {

        //         },
        //         State::Separator => {
        //             match self.prev_state {
        //                 State::Timestamp => {
        //                     next_state = State::NumSat;
        //                 },
        //                 State::NumSat => {
        //                     next_state = State::Satellites;
        //                 },
        //                 State::Satellites => {
        //                     next_state = State::Clock;
        //                 },
        //                 State::Clock => {
        //                     next_state = State::Observation;
        //                 },
        //                 State::LLI => {
        //                     next_state = State::SNR;
        //                 },
        //                 State::SNR => {
        //                     panic!("todo");
        //                 },
        //             }
        //         },
        //     }

        //     self.prev_state = self.state;
        //     self.state = next_state;
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
            ptr: 0,
            wr_ptr: 0,
            numsat: 0,
            sv_ptr: 0,
            buf: [0; 1024],
            first_epoch: true,
            first_clock_diff: true,
            state: State::default(),
            prev_state: State::default(),
            crinex_found: false,
            crinex: CRINEX::default(),
            epoch_diff: TextDiff::new(""),
            clock_diff: NumDiff::<M>::new(0, 3),
            obs_diff: HashMap::new(), // init. late
            buf_ascii: String::with_capacity(128),
            epoch_ascii: String::with_capacity(256),
            epoch_descriptor: String::with_capacity(256),
        }
    }

    /// Returns index of next \n in the buffer, if it exists
    fn next_eol(&self, offset: usize) -> Option<usize> {
        self.buf[offset..].iter().position(|b| *b == b'\n')
    }

    // /// When parsing (algorithm being active)
    // /// any COMMENTS encountered should be passed "as is".
    // /// There is no mean to apply the Hatanaka algorithm to COMMENTS located
    // /// in the file body.
    // /// Returns total size already forwarded
    // fn process_buffered_comments(&mut self, buf: &mut [u8]) -> usize {
    //     let len = buf.len();
    //     let mut wr_size = 0;
    //     while let Some(eol) = self.next_eol(self.ptr) {
    //         if let Ok(ascii) = from_utf8(&self.buf[self.ptr..eol]) {
    //             // can we forward this (entirely ?)
    //             if wr_size < len {
    //                 let ascii = ascii.trim_end();
    //                 if ascii.ends_with("COMMENTS") {
    //                     // consitent with COMMENT:
    //                     //  progress in buffer
    //                     //  forward to user (as is)
    //                     let ascii_len = ascii.len();
    //                     buf[wr_size..ascii_len].copy_from_slice(&self.buf[self.ptr..ascii_len]);
    //                     self.ptr += eol +1;
    //                     wr_size += ascii_len;
    //                 }
    //             }
    //         }
    //     }
    //     wr_size
    // }

    /// In the header section, we need to search for the CRINEX header
    /// and we also have to serve the data "as is": we act like a FIFO.
    /// Once End of Header is specified, this state will be over
    /// and decompression scheme needs to deploy.
    fn read_header(&mut self, buf: &mut [u8]) -> Result<usize> {
        let read_size = buf.len();
        let mut next_state = self.state;

        // if self.ptr > 0 {
        //     // we have some leftovers ready to be exposed
        //     if self.ptr >= read_size {
        //         // We have more than needed: expose and exit: no Read
        //         buf[..read_size].copy_from_slice(&self.buf[self.ptr..read_size]);
        //         self.ptr += read_size;
        //         return Ok(read_size);

        //     } else {
        //         // We don't have enough:
        //         //  1. Expose what we have
        //         //  2. new Read
        //         buf[..self.ptr].copy_from_slice(&self.buf[self.ptr..]);
        //         self.ptr = 0;
        //     }
        // }

        // Fill internal buffer (when it's worth)
        if self.wr_ptr < 1024 - 80 {
            let size = self.reader.read(&mut self.buf[self.wr_ptr..])?;
            if size == 0 {
                // End of stream: notify and exit
                return Ok(0);
            }
            self.wr_ptr += size;
        }

        // Studies (complete) lines (only), one at a time
        // All content that we have correctly studied gets exposed to end user.
        // We do not forward incomplete lines, which facilitates the anlysis and overall process

        let mut rem = self.wr_ptr;
        let mut total = 0;
        let mut processed = 0;

        loop {
            let mut skip = false;

            let end = match self.next_eol(0) {
                Some(offset) => offset + 1,
                None => rem,
            };

            let ascii = from_utf8(&self.buf[..end]).map_err(|_| Error::BodyUtf8Data.to_stdio())?;

            // if end < 79 {
            //     // incomplete line: abort, proceed after next read
            //     println!("incomplete: {}/80", end);
            //     break;
            // }

            println!("CRINEX V{}", self.crinex.version.major); // TODO: debug
            println!("ASCII \"{}\"", &ascii[..end - 1]); // TODO: debug!
            println!(
                "total={}/processed={}/rem={}/END={}\n",
                total, processed, rem, end
            );

            // - Grab CRINEX specs when found
            //   should come first, but we're tolerant
            //   This actually tolerates a missing PROG / DATE specs.. whatever
            //
            // - Support both (yet one is incorrect) CRINEX specs marker
            //   The second one being widespread, most likely not generated
            //   by RNX2CRX historical tool
            //
            // - Exit this state when END OF HEADER is reached
            //
            // - PANIC on invalid CRINEX specs.
            //   Which is actually ideal. It will prevent panic'ing later on in the
            //   decyphering process, which could be hard to debug.
            if ascii.ends_with("CRINEX VERS   / TYPE\n") {
                let version = Version::from_str(&ascii[0..20].trim())
                    .map_err(|_| Error::VersionParsing.to_stdio())?;

                self.crinex.with_version(version);
                self.crinex_found = true;
                skip = true;
            } else if ascii.ends_with("CRINEX VERSION / TYPE\n") {
                let version = Version::from_str(&ascii[0..20].trim())
                    .map_err(|_| Error::VersionParsing.to_stdio())?;

                self.crinex.with_version(version);
                self.crinex_found = true;
                skip = true;
            } else if ascii.ends_with("CRINEX PROG / DATE\n") {
                // self.crinex
                //     .with_prog_date(&ascii[0..60].trim())
                //     .map_err(|_| Error::CrinexParsing.to_stdio())?;

                skip = true;
            } else if ascii.ends_with("END OF HEADER\n") {
                next_state = State::Timestamp; // concludes this step
            }

            if !skip {
                // expose to user (as is =header)
                buf[total..total + end].copy_from_slice(&self.buf[..end]);

                total += end;
            }

            // roll left
            // TODO kind of inefficient to do this everytime
            //     we might be able to improve this later
            //     but it will require two pointers
            self.buf.copy_within(end.., 0);

            processed += end;
            rem -= (end);

            if next_state == State::Timestamp {
                println!("BREAK 1");
                break; // exit: end of this state
            }

            if total >= read_size {
                println!("BREAK 2");
                break; // exit: can't accept more data
            }

            if total + 80 > read_size {
                // would not fit
                println!("BREAK 3");
                break;
            }

            if rem < 80 {
                println!("BREAK 4");
                break; // not enough to form a complete line
            }
        }

        self.state = next_state; // update state
        Ok(total)
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
