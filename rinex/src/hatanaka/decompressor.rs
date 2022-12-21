//! RINEX decompression module
use super::{numdiff::NumDiff, textdiff::TextDiff, Error};
use crate::{is_comment, prelude::*};

use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum State {
    EpochDescriptor,
    ClockOffsetDescriptor,
    Body,
}

impl Default for State {
    fn default() -> Self {
        Self::EpochDescriptor
    }
}

/// Structure to decompress CRINEX data
pub struct Decompressor {
    /// finite state machine
    state: State,
    /// True only for first epoch ever processed
    first_epoch: bool,
    /// Epoch differentiator
    epoch_diff: TextDiff,
    /// recovered but unformatted CRINEX is stored
    /// because it is particularly easy to parse to identify sv.
    /// It still needs to be formatted for the final result though.
    epoch_descriptor: String,
    /// Clock offset differentiator
    clock_diff: NumDiff,
    /// vehicule identification
    sv_ptr: usize,
    nb_sv: usize, // sv_ptr range
    /// Vehicule differentiators
    sv_diff: HashMap<Sv, Vec<(NumDiff, TextDiff, TextDiff)>>,
}

/// Reworks given content to match RINEX specifications
/// of an epoch descriptor
fn format_epoch(
    version: u8,
    nb_sv: usize,
    content: &str,
    clock_offset: Option<i64>,
) -> Result<String, Error> {
    let mut result = String::new();
    match version {
        1 | 2 => {
            // old RINEX
            // append Systems #ID,
            //  on as many lines as needed
            let min_size = 32 + 3; // epoch descriptor + at least one vehicule
            if content.len() < min_size {
                // parsing would fail
                return Err(Error::FaultyRecoveredEpoch);
            }

            let (epoch, systems) = content.split_at(32); // grab epoch
            result.push_str(&epoch.replace("&", " ")); // rework

            //CRINEX has systems squashed in a single line
            // we just split it to match standard definitions
            // .. and don't forget the tab
            if nb_sv <= 12 {
                // fits in a single line
                result.push_str(systems);
                if let Some(value) = clock_offset {
                    result.push_str(&format!("  {:3.9}", (value as f64) / 1000.0_f64))
                }
            } else {
                // does not fit in a single line
                let mut index = 0;
                for i in 0..nb_sv {
                    if index == 12 {
                        index = 0;
                        if i == 12 {
                            // first line,
                            if let Some(value) = clock_offset {
                                result.push_str(&format!("  {:3.9}", (value as f64) / 1000.0_f64))
                            }
                        }
                        // tab indent
                        result.push_str("\n                                "); //TODO: improve this please
                    }
                    /*
                     * avoids overflowing
                     */
                    let min_offset = i * 3;
                    let max_offset = std::cmp::min(min_offset + 3, systems.len());
                    result.push_str(&systems[min_offset..max_offset]);
                    index += 1;
                }
            }
        },
        _ => {
            // Modern RINEX case
            // Systems #ID to be passed on future lines
            if content.len() < 35 {
                // parsing would fail
                return Err(Error::FaultyRecoveredEpoch);
            }
            let (epoch, _) = content.split_at(35);
            result.push_str(&epoch.replace("&", " "));
            //TODO clock offset
            if let Some(value) = clock_offset {
                result.push_str(&format!("         {:3.12}", (value as f64) / 1000.0_f64))
            }
        },
    }
    Ok(result)
}

impl Decompressor {
    /// Creates a new decompression structure
    pub fn new() -> Self {
        Self {
            first_epoch: true,
            state: State::default(),
            epoch_diff: TextDiff::new(),
            epoch_descriptor: String::with_capacity(128),
            clock_diff: NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)
                .expect("failed to prepare compression object"),
            nb_sv: 0,
            sv_ptr: 0,
            sv_diff: HashMap::new(), // init. later
        }
    }
    /*
        fn reset(&mut self) {
            // are we sure this is enough ?
            // special comment markers, like "MERGE" and "SPLICE"
            //  are they to be encountered inside an epoch ?
            self.state = State::default();
        }
    */
    fn parse_nb_sv(content: &str, crx_major: u8) -> Option<usize> {
        let mut offset: usize = 2    // Y
            +2+1 // m
            +2+1 // d
            +2+1 // h
            +2+1 // m
            +11  // s
            +1   // ">" or "&" init marker
            +3; // epoch flag
        if content.starts_with("> ") {
            //CRNX3 initial epoch, 1 extra whitespace
            offset += 1;
        }
        if crx_major > 1 {
            offset += 2; // YYYY on 4 digits
        }

        let (_, rem) = content.split_at(offset);
        let (n, _) = rem.split_at(3);
        if let Ok(n) = u16::from_str_radix(n.trim(), 10) {
            Some(n.into())
        } else {
            None
        }
    }

    fn parse_flags(&mut self, sv: &Sv, content: &str) {
        //println!("FLAGS: \"{}\"", content); // DEBUG
        if let Some(sv_diff) = self.sv_diff.get_mut(sv) {
            for index in 0..content.len() {
                if let Some(sv_obs) = sv_diff.get_mut(index / 2) {
                    if index % 2 == 0 {
                        // LLI
                        let _ = sv_obs.1.decompress(&content[index..index + 1]);
                    } else {
                        //SSI
                        let _ = sv_obs.2.decompress(&content[index..index + 1]);
                    }
                }
            }
        }
    }

    fn current_satellite(
        &self,
        crx_major: u8,
        crx_constellation: &Constellation,
        sv_ptr: usize,
    ) -> Option<Sv> {
        let epoch = &self.epoch_descriptor;
        let offset: usize = match crx_major {
            1 => std::cmp::min((32 + 3 * (sv_ptr + 1)).into(), epoch.len()), // overflow protection
            _ => std::cmp::min((41 + 3 * (sv_ptr + 1)).into(), epoch.len()), // overflow protection
        };
        let system = epoch.split_at(offset.into()).0;
        let (_, svnn) = system.split_at(system.len() - 3); // last 3 XXX
        let svnn = svnn.trim();
        match crx_major > 2 {
            false => {
                // OLD
                match crx_constellation {
                    Constellation::Mixed => {
                        // OLD and MIXED is fine
                        if let Ok(sv) = Sv::from_str(svnn) {
                            Some(sv)
                        } else {
                            None
                        }
                    },
                    constellation => {
                        // OLD + FIXED: constellation might be omitted.......
                        if let Ok(prn) = u8::from_str_radix(&svnn[1..].trim(), 10) {
                            Some(Sv {
                                prn,
                                constellation: *constellation,
                            })
                        } else {
                            None
                        }
                    },
                }
            },
            true => {
                // MODERN
                if let Ok(sv) = Sv::from_str(svnn) {
                    Some(sv)
                } else {
                    None
                }
            },
        }
    }
    /// Decompresses (recovers) RINEX from given CRINEX content.
    /// This method expects either RINEX comments,
    /// or CRNX1/CRNX3 content, that is either epoch description
    /// and epoch content.
    pub fn decompress(
        &mut self,
        crx_major: u8,
        crx_constell: &Constellation,
        rnx_major: u8,
        observables: &HashMap<Constellation, Vec<Observable>>,
        content: &str,
    ) -> Result<String, Error> {
        // content browser
        let mut result: String = String::new();
        let mut lines = content.lines();
        loop {
            // browse all provided lines
            let line: &str = match lines.next() {
                Some(l) => l,
                None => break,
            };

            //println!("DECOMPRESSING - \"{}\"", line); //DEBUG
            //println!("state: {:?}", self.state);

            // [0] : COMMENTS (special case)
            if is_comment!(line) {
                //if line.contains("RINEX FILE SPLICE") {
                // [0*] SPLICE special comments
                //      merged RINEX Files
                //    self.reset();
                //}
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push_str("\n");
                continue; // move to next line
            }

            // [0*]: special epoch events
            //       with uncompressed descriptor
            //       (CRNX3)
            if line.starts_with("> ") && !self.first_epoch {
                result // feed content as is
                    .push_str(line);
                result // \n dropped by .lines()
                    .push_str("\n");
                continue; // move to next line
            }

            match self.state {
                State::EpochDescriptor => {
                    if self.first_epoch {
                        match crx_major {
                            1 => {
                                if !line.starts_with("&") {
                                    return Err(Error::FaultyCrx1FirstEpoch);
                                }
                            },
                            3 => {
                                if !line.starts_with(">") {
                                    return Err(Error::FaultyCrx3FirstEpoch);
                                }
                            },
                            _ => {}, // will never happen
                        }

                        // Kernel initialization,
                        // only once, always text based
                        // from this entire line
                        self.epoch_diff.init(line.trim_end());
                        self.first_epoch = false;
                    } else {
                        /*
                         * this latches the current line content
                         * we'll deal with it when combining with clock offsets
                         */
                        self.epoch_diff.decompress(line);
                    }

                    self.state = State::ClockOffsetDescriptor;
                }, // state::EpochDescriptor

                State::ClockOffsetDescriptor => {
                    /*
                     * this line is dedicated to clock offset description
                     */
                    let mut clock_offset: Option<i64> = None;
                    if line.contains("&") {
                        // clock offset kernel (re)init
                        let (n, rem) = line.split_at(1);
                        if let Ok(order) = u8::from_str_radix(n, 10) {
                            let (_, value) = rem.split_at(1);
                            if let Ok(value) = i64::from_str_radix(value, 10) {
                                self.clock_diff.init(order.into(), value)?;
                            } else {
                                return Err(Error::ClockOffsetValueError);
                            }
                        } else {
                            return Err(Error::ClockOffsetOrderError);
                        }
                    } else {
                        // --> nominal clock offset line
                        if let Ok(value) = i64::from_str_radix(line.trim(), 10) {
                            clock_offset = Some(value); // latch for later
                        }
                    }

                    /*
                     * now we have all information to reconstruct the epoch descriptor
                     */
                    let recovered = self.epoch_diff.decompress(" ").trim_end();
                    // we store the recovered and unformatted CRINEX descriptor
                    //   because it is particularly easy to parse,
                    //   as it is made of a single line.
                    //   It needs to be formatted according to standards,
                    //   for the result being constructed. See the following operations
                    self.epoch_descriptor = recovered.clone().to_string();
                    // initialize sv identifier
                    self.sv_ptr = 0;
                    if let Some(n) = Self::parse_nb_sv(&self.epoch_descriptor, crx_major) {
                        self.nb_sv = n;
                    } else {
                        return Err(Error::VehiculeIdentificationError);
                    }

                    if let Ok(descriptor) =
                        format_epoch(rnx_major, self.nb_sv, recovered, clock_offset)
                    {
                        //println!("--- EPOCH --- \n{}[STOP]", descriptor.trim_end()); //DEBUG
                        result.push_str(&format!("{}\n", descriptor.trim_end()));
                    } else {
                        return Err(Error::EpochConstruct);
                    }

                    self.state = State::Body;
                }, // state::ClockOffsetDescriptor

                State::Body => {
                    let mut obs_ptr: usize = 0;
                    let mut observations: Vec<Option<i64>> = Vec::with_capacity(16);
                    /*
                     * identify satellite we're dealing with
                     */
                    if let Some(sv) = self.current_satellite(crx_major, &crx_constell, self.sv_ptr)
                    {
                        //println!("SV: {:?}", sv); //DEBUG
                        self.sv_ptr += 1; // increment for next time
                                          // vehicules are always described in a single line
                        if rnx_major > 2 {
                            // RNX3 needs SVNN on every line
                            result.push_str(&format!("{} ", sv));
                        }
                        /*
                         * Build compress tools in case this vehicule is new
                         */
                        if self.sv_diff.get(&sv).is_none() {
                            let mut inner: Vec<(NumDiff, TextDiff, TextDiff)> =
                                Vec::with_capacity(16);
                            // this protects from malformed Headers or malformed Epoch descriptions
                            if let Some(codes) = observables.get(&sv.constellation) {
                                for _ in codes {
                                    let mut kernels = (
                                        NumDiff::new(NumDiff::MAX_COMPRESSION_ORDER)?,
                                        TextDiff::new(),
                                        TextDiff::new(),
                                    );
                                    kernels.1.init(" "); // LLI
                                    kernels.2.init(" "); // SSI
                                    inner.push(kernels);
                                }
                                self.sv_diff.insert(sv, inner);
                            }
                        }
                        /*
                         * iterate over entire line
                         */
                        let mut line = line.trim_end();
                        if let Some(codes) = observables.get(&sv.constellation) {
                            while obs_ptr < codes.len() {
                                if let Some(pos) = line.find(' ') {
                                    let content = &line[..pos];
                                    //println!("OBS \"{}\" - CONTENT \"{}\"", codes[obs_ptr], content); //DEBUG
                                    if content.len() == 0 {
                                        /*
                                         * missing observation
                                         */
                                        observations.push(None);
                                    } else {
                                        /*
                                         * regular progression
                                         */
                                        if let Some(sv_diff) = self.sv_diff.get_mut(&sv) {
                                            if let Some(marker) = content.find("&") {
                                                // kernel (re)initialization
                                                let (order, rem) = content.split_at(marker);
                                                let order = u8::from_str_radix(order.trim(), 10)?;
                                                //println!("ORDER {}", order); //DEBUG
                                                let (_, data) = rem.split_at(1);
                                                if let Ok(data) =
                                                    i64::from_str_radix(data.trim(), 10)
                                                {
                                                    sv_diff[obs_ptr]
                                                        .0 // observations only, at this point
                                                        .init(order.into(), data)?;
                                                    observations.push(Some(data));
                                                }
                                            } else {
                                                // regular compression
                                                if let Ok(num) =
                                                    i64::from_str_radix(content.trim(), 10)
                                                {
                                                    let recovered = sv_diff[obs_ptr]
                                                        .0 // observations only, at this point
                                                        .decompress(num);
                                                    observations.push(Some(recovered));
                                                }
                                            }
                                        }
                                    }
                                    line = &line[std::cmp::min(pos + 1, line.len())..]; // line remainder
                                    obs_ptr += 1;
                                } else {
                                    /*
                                     * EOL detected, but obs_ptr < codes.len()
                                     *  => try to parse one last obs
                                     */
                                    //println!("OBS \"{}\" - CONTENT \"{}\"", codes[obs_ptr], line); //DEBUG
                                    if let Some(sv_diff) = self.sv_diff.get_mut(&sv) {
                                        if let Some(marker) = line.find("&") {
                                            // kernel (re)initliaization
                                            let (order, rem) = line.split_at(marker);
                                            let order = u8::from_str_radix(order.trim(), 10)?;
                                            let (_, data) = rem.split_at(1);
                                            if let Ok(data) = i64::from_str_radix(data.trim(), 10) {
                                                sv_diff[obs_ptr]
                                                    .0 // observations only, at this point
                                                    .init(order.into(), data)?;
                                                observations.push(Some(data));
                                            }
                                        } else {
                                            // regular compression
                                            if let Ok(num) = i64::from_str_radix(line.trim(), 10) {
                                                let recovered = sv_diff[obs_ptr]
                                                    .0 // observations only, at this point
                                                    .decompress(num);
                                                observations.push(Some(recovered))
                                            }
                                        }
                                    } //svdiff
                                    line = ""; // avoid flags parsing: all flags omitted <=> content unchanged
                                    obs_ptr = codes.len();
                                } //EOL
                            } //while()
                        } //observables identification
                          /*
                           * Flags field
                           */
                        if line.len() > 0 {
                            // can parse at least 1 flag
                            self.parse_flags(&sv, line);
                        }
                        /*
                         * group previously parsed observations,
                         *   into a single formatted line
                         *   or into several in case of OLD RINEX
                         */
                        for (index, data) in observations.iter().enumerate() {
                            if let Some(data) = data {
                                let sv_diff = self.sv_diff.get_mut(&sv).unwrap(); //cant fail at this point
                                let lli = sv_diff[index]
                                    .1 // LLI
                                    .decompress(" ") // trick to recover
                                    // using textdiff property.
                                    // Another option would be to have an array to
                                    // store them
                                    .to_string();
                                let ssi = sv_diff[index]
                                    .2 // SSI
                                    .decompress(" ") // trick to recover
                                    // using textdiff property.
                                    // Another option would be to have an array to
                                    // store them
                                    .to_string();
                                result.push_str(&format!(
                                    "{:13.3}{}{} ",
                                    *data as f64 / 1000_f64,
                                    lli,
                                    ssi
                                )); //F14.3
                            } else {
                                result.push_str("                "); // BLANK
                            }

                            if rnx_major < 3 {
                                // old RINEX
                                if (index + 1).rem_euclid(5) == 0 {
                                    // maximal nb of OBS per line
                                    result.push_str("\n")
                                }
                            }
                        }
                        //result.push_str("\n");
                    }
                    // end of line parsing
                    //  if sv_ptr has reached the expected amount of vehicules
                    //  we reset to state (1)
                    if self.sv_ptr >= self.nb_sv {
                        self.state = State::EpochDescriptor;
                    }
                }, //current_satellite()
            } //match(state)
        } //loop
          //println!("--- TOTAL DECOMPRESSED --- \n\"{}\"", result); //DEBUG
        Ok(result)
    }
}
