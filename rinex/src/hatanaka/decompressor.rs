//! CRINEX decompression module
use crate::{
    hatanaka::{Error, NumDiff, ObsDiff, TextDiff, CRINEX},
    prelude::{Constellation, Epoch, EpochFlag, Header, Observable, Version, SV},
};

use std::{
    collections::HashMap,
    io::{Read, Result},
    str::{from_utf8, FromStr},
};

#[cfg(feature = "log")]
use log::debug;

#[cfg(docsrs)]
use crate::hatanaka::Compressor;

// TODO
// The current implementation is not really flexible towards invalid CRINEX.
// Which is totally fine ! but high level applications could benefit a little flexibility ?
//
// ** search for .unwrap() and .expect() **
// Numsat tolerance and SV tolerance could be added
// we could introduce a Garbage state and simply trash incoming data until End of Epoch.
// But exiting that state is not easy !
#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub enum State {
    #[default]
    /// Expecting "CRINEX VERSION / TYPE"
    Version,
    /// Expecting "CRINEX PROG / DATE"
    ProgDate,
    /// Waiting for observable specifications
    HeaderSpecs,
    /// Waiting for "END OF HEADER" special marker
    EndofHeader,
    /// [State::NewEpoch] preceeds [State::Timestamp]
    NewEpoch,
    /// About to decode an [Epoch]
    Timestamp,
    /// About to decode [EpochFlag]
    EpochFlag,
    /// Reading Numsat
    NumSat,
    /// Reading Satellites
    Satellites,
    /// Reading Clock
    Clock,
    /// Observation
    Observation,
    /// LLI
    LLI,
    /// SNR
    SNR,
    /// Gathering a whitespace Separator
    Separator,
    /// Gathering a line termination
    EOL,
}

impl State {
    /// Returns true if we're inside the File Body.
    /// Use this to grab the [CRINEX] definition if you have to.
    pub fn file_body(&self) -> bool {
        !matches!(
            self,
            Self::Version | Self::ProgDate | Self::EndofHeader | Self::HeaderSpecs
        )
    }

    pub fn line_terminated(&self) -> bool {
        matches!(
            self,
            Self::Version | Self::ProgDate | Self::EndofHeader | Self::Clock | Self::HeaderSpecs
        )
    }

    /// CRINEX specs are not forwarded & kept internally
    pub fn skip(&self) -> bool {
        matches!(self, Self::Version | Self::ProgDate)
    }

    /// Returns expected fixed size, when it applies.
    /// For simpler [State]s we know the expected amount of bytes ahead of time.
    /// For complex [State]s it is impossible to predetermine.
    pub fn fixed_size(&self, v3: bool, numsat: usize) -> Option<usize> {
        match self {
            Self::NewEpoch => Some(1),
            Self::Version => Some(80),
            Self::ProgDate => Some(78),
            Self::Timestamp => {
                if v3 {
                    Some(28) // YEAR on 4 digits
                } else {
                    Some(26) // YEAR on 2 digits
                }
            },
            Self::EpochFlag => Some(3),
            Self::NumSat => Some(3),
            Self::Observation => Some(14),
            Self::Satellites => Some(3 * numsat),
            Self::LLI | Self::SNR | Self::Separator => Some(1),
            _ => None,
        }
    }
}

/// [Decompressor] is a structure to decompress CRINEX data effeciently.
/// The decoding process may panic in a few circumstances:
///   - numsat encoding is faulty (invalid digit).
///   We could improve that by introducing a Garbage state.
///   - the recovered epoch contains invalid SV descriptiong (bad data)
///   - Header stream does not describe GNSS systems (rubbish CRINEX).
///   - each observation must contain valid digits (bad data)
///   We could improve that by introducing a Garbage state.
///
/// [Decompressor] is very powerful and will remove trailing whitespaces,
/// which facilitates the parsing process to come right after decompression.
///
/// You can use the [Decompressor] in your own applications, especially
/// noting that it can work on any [Read]able I/O interface: it does not have to
/// work on local files !
///
/// When building [Decompressor] you need to specify
/// the absolute maximal compression level M to be supported.
/// M=5 is hardcoded in the historical CRX2RNX tool, if you're coming from this tool
/// you should use this value.
/// ```
/// use std::fs::File;
/// use rinex::hatanaka::Decompressor;
///
/// // Working from local files is the typical application,
/// // but [Decompressor] may deploy over any [Read]able interface
/// let mut fd = File::open("../test_resources/CRNX/V1/AJAC3550.21D")
///     .unwrap();
///
/// // This file was compressed using the historical tool, M=5 limit is OK.
/// let decomp = Decompressor::<5>new(fd);
///
/// // Dump this as a new (readable) RINEX
/// let mut total = 0;
/// let mut buf = Vec::<u8>::with_capacity(1024);
/// while let Some(size) = decomp.read(&mut buf) {
///     if size == 0 {
///         break; // EOS reached
///     }
///     total += size;
/// }
///
/// assert_eq!(total, 36); // total bytewise
/// ```
///
/// If you compressed the data yourself, especially working with our [Compressor],
/// you have complete control over the compression level. But you have to understand
/// that CRINEX is not a lossless compression and M=5 is said to be the optimal compromise.
///
/// ```
/// use std::fs::File;
/// use rinex::{
///     prelude::RINEX,
///     hatanaka::{Compressor, Decompressor},
/// };
///
/// let mut fd = File::open("../test_resources/OBS/V1/AJAC3550.21D")
///     .unwrap();
///
/// let mut comp = Compressor::<5>::new(fd);
///
/// // compress some RINEX to CRINEX
/// let mut buf = Vec::<u8>::with_capacity(1024);
/// while let Some(size) = fd.read(&mut buf) {
///     
/// }
///
/// ```
pub struct Decompressor<const M: usize, R: Read> {
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
    /// For internal logic
    prev_state: State,
    /// CRINEX header that we did identify.
    /// You should wait until State != State::InsideHeader to consider this field.
    pub crinex: CRINEX,
    /// [Read]able interface
    reader: R,
    /// Internal buffer
    buf: [u8; 4096],
    /// Pointers
    ptr: usize,
    avail: usize,
    eos: bool,
    /// pointers
    numsat: usize, // total
    sv_ptr: usize, // inside epoch
    sv: SV,
    /// pointers
    numobs: usize, // total
    obs_ptr: usize, // inside epoch
    /// [TextDiff] that works on entire Epoch line
    epoch_diff: TextDiff,
    /// Epoch descriptor, for single allocation
    epoch_descriptor: String,
    /// Clock offset differentiator
    clock_diff: NumDiff<M>,
    /// Observation differentiators
    obs_diff: HashMap<(SV, usize), ObsDiff<M>>,
    /// [Constellation] described in Header field
    constellation: Constellation,
    /// [Observable]s specs for each [Constellation]
    gnss_observables: HashMap<Constellation, Vec<Observable>>,
}

impl<const M: usize, R: Read> Read for Decompressor<M, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // always try to fill internal buffer
        if self.avail < 4096 {
            let size = self.reader.read(&mut self.buf[self.avail..])?;
            self.eos = size == 0;
            self.avail += size;
        }

        let buf_len = buf.len();
        let mut total = 0;

        // Run FSM
        loop {
            let mut force_skip = false;
            let mut next_state = self.state;

            let numsat = self.numsat;
            let v3 = self.crinex.version.major == 3;

            let mut min_size = self.state.fixed_size(v3, numsat);

            let sv_constell = self.sv.constellation; // constellation of current SV
            let sv_observables = self.gnss_observables.get(&sv_constell);

            // complex states for which data quantity is not known ahead of time
            if min_size.is_none() {
                match self.state {
                    State::HeaderSpecs | State::EndofHeader | State::Clock => {
                        // Header pass -through:
                        //  all we know is the line is at least 60 byte long and \n terminated.
                        //  Here we're waiting for complete lines to pass through & proceed
                        if self.avail > 120 {
                            // RINEX header lines will never be this long, this is almost 2 complete lines
                            if let Some(eol) =
                                Self::find(&self.buf[..120], Self::END_OF_LINE_TERMINATOR)
                            {
                                min_size = Some(eol + 1);
                            }
                        }
                    },
                    State::EOL => panic!("eol"),
                    others => unreachable!("{:?}", others),
                }
            }

            if min_size.is_none() {
                // end of pointer has not been resolved: we can't proceed
                // because buffer does not contain enought data.
                if total > 0 {
                    return Ok(total); // expose what we did process though
                } else {
                    return Err(Error::NeedMoreData.to_stdio()); // we need more data from I/O
                }
            }

            let mut min_size = min_size.unwrap();

            // special case for obs payload
            if self.state == State::Observation {
                if self.obs_ptr > 0 {
                    min_size += 2;
                }
            }

            #[cfg(feature = "log")]
            println!(
                "[V{}] {:?} avail={}/total={}",
                self.crinex.version.major, self.state, self.avail, total
            );

            let ascii = from_utf8(&self.buf[..min_size])
                .map_err(|_| Error::BadUtf8Data.to_stdio())?
                .trim_end();

            let ascii_len = ascii.len();
            let bytes = ascii.as_bytes();

            #[cfg(feature = "log")]
            println!("ASCII: \"{}\" [{}]", ascii, ascii_len);

            // next step would not fit in user buffer
            // exit: return processed data
            if total + ascii_len + 1 > buf_len {
                println!("early break: would not fit");
                return Ok(total);
            }

            match self.state {
                State::Version => {
                    let version = Version::from_str(&ascii[0..20].trim())
                        .map_err(|_| Error::VersionParsing.to_stdio())?;

                    self.crinex = self.crinex.with_version(version);
                    next_state = State::ProgDate;
                },
                State::ProgDate => {
                    self.crinex = self
                        .crinex
                        .with_prog_date(&ascii.trim_end())
                        .map_err(|_| Error::CrinexParsing.to_stdio())?;

                    next_state = State::HeaderSpecs;
                },
                State::HeaderSpecs => {
                    if ascii.ends_with("END OF HEADER") {
                        panic!("bad crinex: no observation specs");
                    }
                    if ascii.contains("TYPES OF OBS") {
                        // V2 special marker
                        if v3 {
                            panic!("bad crinex: invalid obs specs");
                        }

                        // observable identification
                        if self.numobs == 0 {
                            // first encounter ever
                            if let Ok(numobs) = ascii[..10].trim().parse::<u8>() {
                                self.numobs = numobs as usize;
                            }
                        }

                        // identify all values for this line
                        let num = ascii_len / 6;

                        for i in 0..num {
                            let start = i * 6;
                            let end = start + 6;
                            if let Ok(obs) = Observable::from_str(&ascii[start..end]) {
                                println!("found {}", obs);

                                // store for later
                                if let Some(specs) =
                                    self.gnss_observables.get_mut(&self.constellation)
                                {
                                    specs.push(obs);
                                } else {
                                    self.gnss_observables.insert(self.constellation, vec![obs]);
                                }

                                self.obs_ptr += 1;
                            }
                        }

                        if self.obs_ptr == self.numobs {
                            // done parsing all specs
                            self.obs_ptr = 0; // reset for later
                            self.numobs = 0; // reset for later
                            next_state = State::EndofHeader;
                        }
                    }
                    if ascii.contains("SYS / # / OBS TYPES") {
                        if !v3 {
                            panic!("bad crinex: invalid obs specs");
                        }

                        // observable identification
                        if self.numobs == 0 {
                            // first encounter ever
                            if let Ok(numobs) = ascii[..10].trim().parse::<u8>() {
                                self.numobs = numobs as usize;
                            }
                        }
                    }
                },
                State::EndofHeader => {
                    if ascii.ends_with("END OF HEADER") {
                        println!("==== END OF HEADER ====");
                        println!("Specs: {:?}", self.gnss_observables);

                        next_state = State::NewEpoch;
                        self.epoch_descriptor.clear(); // prepare for decoding
                    }
                },
                State::NewEpoch => {
                    if ascii[0..1].eq("&") {
                        // TODO: core reset
                        println!("core reset!");
                    }

                    self.epoch_descriptor.push_str(ascii);
                    next_state = State::Timestamp;
                },
                State::Timestamp => {
                    self.epoch_descriptor.push_str(ascii);
                    next_state = State::EpochFlag;
                },
                State::EpochFlag => {
                    let flag = ascii.trim().parse::<EpochFlag>().expect("bad flag data");

                    next_state = State::NumSat;
                    self.epoch_descriptor.push_str(ascii);
                },
                State::NumSat => {
                    let numsat = ascii.trim().parse::<u8>().unwrap();

                    self.numsat = numsat as usize;
                    self.sv_ptr = 0;
                    next_state = State::Satellites;
                    self.epoch_descriptor.push_str(ascii);
                },
                State::Satellites => {
                    next_state = State::Clock;

                    self.epoch_descriptor.push_str(ascii);

                    // we've gathered the complete descriptor: uncompress it
                    self.epoch_descriptor = self
                        .epoch_diff
                        .decompress(&self.epoch_descriptor)
                        .to_string();

                    println!("recovered descriptor: \"{}\"", self.epoch_descriptor);

                    self.sv = SV::from_str(&self.epoch_descriptor[32..32 + 3])
                        .expect("bad sv description");

                    println!("first sv: {}", self.sv);
                    self.sv_ptr = 0;
                    self.obs_ptr = 0;
                },
                State::Clock => {
                    if ascii.len() == 0 {
                        next_state = State::Observation;
                    } else {
                        unimplemented!("compressed clock decoding!");
                    }

                    // identify specs for next SV
                    if let Some(codes) = self.gnss_observables.get(&Constellation::GPS) {
                        self.numobs = codes.len();
                    } else {
                        panic!("missing specs for {}", Constellation::GPS);
                    }
                },
                State::Observation => {
                    println!("sv_ptr={}/{}", self.sv_ptr, self.numsat);
                    println!("obs_ptr={}/{}", self.obs_ptr, self.numobs);

                    let level = ascii[0..1]
                        .parse::<u8>()
                        .expect("invalid compression level")
                        as usize;

                    let reset = ascii[1..2].eq("&");

                    let val = ascii[2..].trim().parse::<i64>().expect("invalid i64 data");

                    if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)) {
                        if reset {
                            kernel.data_diff.force_init(val, level);
                        } else {
                            kernel.data_diff.decompress(val);
                        }
                    } else {
                        let kernel = ObsDiff::<M>::new(val, level, "", "");
                        self.obs_diff.insert((self.sv, self.obs_ptr), kernel);
                    }

                    self.obs_ptr += 1;

                    if self.obs_ptr == self.numobs {
                        self.obs_ptr = 0;
                        next_state = State::LLI;
                    } else {
                        next_state = State::Separator;
                    }
                },
                State::LLI => {
                    let kernel = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)).unwrap();

                    kernel.lli_diff.decompress(&ascii[..1]);

                    next_state = State::SNR;
                },
                State::SNR => {
                    let kernel = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)).unwrap();

                    kernel.snr_diff.decompress(&ascii[..1]);

                    self.obs_ptr += 1;
                    if self.obs_ptr == self.numobs {
                        self.obs_ptr = 0;
                        self.sv_ptr += 1;
                        if self.sv_ptr == self.numsat {
                            self.sv_ptr = 0;
                            next_state = State::Timestamp;
                        } else {
                            next_state = State::Observation;
                        }
                    } else {
                        next_state = State::Observation;
                    }
                },
                State::Separator => match self.prev_state {
                    State::Observation => {
                        next_state = State::Observation;

                        if self.obs_ptr == self.numobs {
                            self.sv_ptr += 1;
                            self.obs_ptr = 0;
                            if self.sv_ptr == self.numsat {
                                self.sv_ptr = 0;
                                next_state = State::Timestamp;
                            }
                        }
                    },
                    others => unreachable!("{:?}", others),
                },
                _ => {
                    panic!("INVALID!");
                },
            }

            // content is passed on to user
            if !self.state.skip() {
                buf[total..total + ascii_len].copy_from_slice(&bytes[..ascii_len]);
                buf[total + ascii_len] = b'\n'; // \n
                total += ascii_len + 1;
            }

            let mut end = ascii_len;
            if self.state.line_terminated() {
                end += 1; // discard EOL
            }

            // special case: empty clock (omitted)
            if self.state == State::Clock && ascii_len == 0 {
                end += 1;
            }

            // roll left
            // TODO : this is time consuming
            //        we should move on to a double pointer scheme ?
            self.buf.copy_within(end.., 0);
            self.avail -= end;
            self.prev_state = self.state;
            self.state = next_state;
        }

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
    /// EOL is used in the decoding process
    const END_OF_LINE_TERMINATOR: u8 = b'\n';

    // /// Max. number of [Observable]s described in a single [State::HeaderSpecs] line
    // const NAX_SPECS_PER_LINE :usize = 9;

    /// Locates byte in given slice
    fn find(slice: &[u8], byte: u8) -> Option<usize> {
        slice.iter().position(|b| *b == byte)
    }

    /// Creates a new [Decompressor] working from [Read]
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            ptr: 0,
            avail: 0,
            numsat: 0,
            sv_ptr: 0,
            numobs: 0,
            obs_ptr: 0,
            buf: [0; 4096],
            eos: false,
            sv: Default::default(),
            state: Default::default(),
            prev_state: Default::default(),
            crinex: Default::default(),
            epoch_diff: TextDiff::new(""),
            clock_diff: NumDiff::<M>::new(0, M),
            obs_diff: HashMap::with_capacity(8), // cannot be initialized yet
            epoch_descriptor: String::with_capacity(256),
            constellation: Constellation::default(),
            gnss_observables: HashMap::with_capacity(4),
        }
    }

    // /// Returns index of next \n in the buffer, if it exists
    // fn next_eol(&self, offset: usize) -> Option<usize> {
    //     self.buf[offset..].iter().position(|b| *b == b'\n')
    // }

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

#[cfg(test)]
mod test {
    use super::Decompressor;
    use std::{fs::File, io::Read};

    #[test]
    fn test_decompress_v1() {
        // Test the lowest level Decompressor seamless reader.
        // Other testing then move on to tests::reader (BufferedReader) which provides
        //  .lines() Iteratation, which is more suited for indepth testing and easier interfacing.
        let fd = File::open("../test_resources/CRNX/V1/AJAC3550.21D").unwrap();

        let mut decompressor = Decompressor::<5, File>::new(fd);

        let mut buf = Vec::<u8>::with_capacity(100000);

        let size = decompressor.read_to_end(&mut buf).unwrap();

        assert!(size > 0);

        let string =
            String::from_utf8(buf).expect("decompressed CRINEX does not containt valid utf8");
    }
}
