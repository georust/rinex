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

/// Locates byte in given slice
fn find(slice: &[u8], byte: u8) -> Option<usize> {
    slice.iter().position(|b| *b == byte)
}

/// Special observation locator
fn find_end_of_observation(slice: &[u8]) -> Option<usize> {
    let mut offset = Option::<usize>::None;
    for (nth, byte) in slice.iter().enumerate() {
        if *byte == b' ' {
            offset = Some(nth);
        } else {
            if offset.is_some() {
                return offset;
            }
        }
    }
    offset
}

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
    /// In [State::EpochGathering] we're trying to gather a complete
    /// Epoch description.
    EpochGathering,
    /// In [State::EpochGathered] we actually decode the descriptor we just
    /// gathered or recovered.
    EpochGathered,
    /// Trying to gather a complete observation
    Observation,
    /// LLI
    LLI,
    /// SNR
    SNR,
    /// Gathering a line termination
    EOL,
}

impl State {
    /// Minimal size of a valid [Epoch] description in V2 revision    
    /// - Timestamp: Year uses 2 digits
    /// - Flag
    /// - Numsat
    const MIN_V2_EPOCH_DESCRIPTION_SIZE: usize = 26 + 3 + 3;

    /// Minimal size of a valid [Epoch] description in V3 revision  
    /// - Timestamp: Year uses 4 digits
    /// - Flag
    /// - Numsat
    const MIN_V3_EPOCH_DESCRIPTION_SIZE: usize = 28 + 3 + 3;

    /// Returns true if we're inside the File Body.
    /// Use this to grab the [CRINEX] definition if you have to.
    pub fn file_body(&self) -> bool {
        !matches!(
            self,
            Self::Version | Self::ProgDate | Self::EndofHeader | Self::HeaderSpecs
        )
    }

    /// Returns true if we're inside the File Header.
    /// Use this to determine it is not safe to grab the [CRINEX] definition yet.
    pub fn file_header(&self) -> bool {
        !self.file_body()
    }

    /// True if this [State] is \n terminated
    fn line_terminated(&self) -> bool {
        matches!(
            self,
            Self::Version
                | Self::ProgDate
                | Self::EndofHeader
                | Self::HeaderSpecs
                | Self::EpochGathered
                | Self::EpochGathering
        )
    }

    /// True if the current buffer content should not be forwarded to user.
    pub fn skipped(&self) -> bool {
        matches!(self, Self::Version | Self::ProgDate | Self::EpochGathering)
    }

    /// Returns expected fixed size, when it applies.
    /// For simpler [State]s we know the expected amount of bytes ahead of time.
    /// For complex [State]s it is impossible to predetermine.
    pub fn fixed_size(&self) -> Option<usize> {
        match self {
            Self::Version => Some(80),
            Self::ProgDate => Some(78),
            Self::LLI | Self::SNR => Some(1),
            Self::EpochGathered => Some(1),
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
    /// For internal logic: remains true until
    /// first [Epoch] description was decoded.
    first_epoch: bool,
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
    /// Helps identifying BLANK fields
    last_was_blank: bool,
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
            let mut next_state = self.state;
            let mut skipped = self.state.skipped();

            let numsat = self.numsat;
            let v3 = self.crinex.version.major == 3;

            let mut min_size = self.state.fixed_size();

            let sv_constell = self.sv.constellation; // constellation of current SV
            let sv_observables = self.gnss_observables.get(&sv_constell);

            // complex states for which data quantity is not known ahead of time
            if min_size.is_none() {
                match self.state {
                    State::HeaderSpecs | State::EndofHeader => {
                        // Header pass -through:
                        //  all we know is the line is at least 60 byte long and \n terminated.
                        //  Here we're waiting for complete lines to pass through & proceed
                        if self.avail < 128 {
                            // protection not to attempt for nothing
                            // RINEX header lines will never be this long (this is most certainly 2 complete lines)
                            continue;
                        }

                        if let Some(eol) = find(&self.buf[..128], Self::END_OF_LINE_TERMINATOR) {
                            min_size = Some(eol + 1);
                        }
                    },
                    State::EpochGathering => {
                        // Once first epoch is decoded we cannot predict the expected length
                        // and it might actually be kind of small on big compression factor
                        // and encoders that remove tailing white spaces.
                        // The only option is to search for EOL
                        if self.avail < 256 {
                            // protection not to attempt for nothing
                            continue;
                        }
                        if let Some(eol) = find(&self.buf, Self::END_OF_LINE_TERMINATOR) {
                            min_size = Some(eol + 1);
                        }
                    },
                    State::Observation => {
                        // collecting next observation
                        if let Some(wsp) = find(&self.buf, Self::WHITESPACE_BYTE) {
                            min_size = Some(wsp + 1);
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

            let min_size = min_size.unwrap();

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

            let mut roll_start = ascii_len;

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
                        // Reached END_OF_HEADER and HeaderSpecs were not identified.
                        // We would not be able to proceed to decode data.
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

                        next_state = State::EpochGathering;
                        self.epoch_descriptor.clear(); // prepare for decoding
                    }
                },
                State::EpochGathering => {
                    if self.first_epoch {
                        self.epoch_diff.force_init(&ascii);
                        self.epoch_descriptor = ascii.to_string();
                    } else {
                        if v3 {
                            if ascii[..1].eq(">") {
                                // V3 kernel reset
                                self.epoch_diff.force_init(&ascii);
                                self.epoch_descriptor = ascii.to_string();
                            }
                        } else {
                            if ascii[..1].eq("&") {
                                // V2 kernel reset
                                self.epoch_diff.force_init(&ascii);
                                self.epoch_descriptor = ascii.to_string();
                            }
                        }
                    }

                    next_state = State::EpochGathered;
                },
                State::EpochGathered => {
                    // Trying to interprate what is latched in epoch descriptor
                    // any failures lead back to previous state
                    if self.first_epoch {
                        if v3 {
                            // line must start with '>'
                            if !self.epoch_descriptor[..1].eq(">") {
                                self.state = State::EpochGathering;
                                continue;
                            }
                        } else {
                            // line must start with '&'
                            if !self.epoch_descriptor[..1].eq("&") {
                                self.state = State::EpochGathering;
                                continue;
                            }
                        }
                    }

                    // note that we do not verify timestamp correctness
                    // it is not required by the decompression algorithm

                    // decode numsat
                    let numsat_slice = if v3 {
                        &self.epoch_descriptor
                            [Self::V3_TIMESTAMP_SIZE + 3..Self::V3_TIMESTAMP_SIZE + 6]
                    } else {
                        &self.epoch_descriptor
                            [Self::V2_TIMESTAMP_SIZE + 3..Self::V2_TIMESTAMP_SIZE + 6]
                    };

                    let numsat = numsat_slice
                        .trim()
                        .parse::<u8>()
                        .expect("invalid numsat (bad digits)")
                        as usize;

                    self.numsat = numsat;
                    println!("NUMSAT: {}", numsat);

                    // verify satellites correctness, this avoids proceeding to epoch decoding for nothing
                    for i in 0..numsat {
                        let start = if v3 {
                            Self::V3_TIMESTAMP_SIZE + 6 + 3 * i
                        } else {
                            Self::V2_TIMESTAMP_SIZE + 6 + 3 * i
                        };

                        let sv = SV::from_str(&self.epoch_descriptor[start..start + 3].trim())
                            .expect("bad sv");

                        if i == 0 {
                            self.sv = sv;
                        }
                    }

                    // TODO: clock state decompression !

                    // move on to first observation
                    // Here we might have a problem if the first field is already BLANKED OBS
                    // and this work around may fail in case of clock presence: TODO double check

                    self.sv_ptr = 0;
                    self.obs_ptr = 0;
                    self.first_epoch = false;
                    self.last_was_blank = false;

                    self.numobs = self
                        .gnss_observables
                        .get(&self.sv.constellation)
                        .expect("missing observables")
                        .len();

                    next_state = State::Observation;
                },
                State::Observation => {
                    println!("sv_ptr={}/{}", self.sv_ptr, self.numsat);
                    println!(
                        "obs_ptr={}/{} [{}]",
                        self.obs_ptr, self.numobs, self.last_was_blank
                    );

                    if ascii_len == 0 {
                        if self.last_was_blank {
                            // BLANK field spotted: consume & move on to next
                            self.obs_ptr += 1;
                        }

                        roll_start += 1; // consume
                        self.last_was_blank = true;
                    } else {
                        // parse value
                        let offset = ascii.find(Self::KERNEL_RESET_MARKER_CHAR);

                        if let Some(offset) = offset {
                            // observation kernel (re-)initialization
                            let level = ascii[0..offset]
                                .trim()
                                .parse::<u8>()
                                .expect("bad compression level digit")
                                as usize;

                            let val = ascii[offset + 1..].trim().parse::<i64>().expect("bad i64");

                            if let Some(kernel) = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)) {
                                kernel.data_diff.force_init(val, level);
                            } else {
                                let kernel = ObsDiff::<M>::new(val, level, "", "");
                                self.obs_diff.insert((self.sv, self.obs_ptr), kernel);
                            }

                            let val = val as f64 / 1000.0;
                        } else {
                            // regular decompression

                            let val = ascii.trim().parse::<i64>().expect("bad i64");

                            let kernel = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)).unwrap();

                            let val = kernel.data_diff.decompress(val);
                            let val = val as f64 / 1000.0;
                        }

                        self.obs_ptr += 1;
                        self.last_was_blank = false;
                    }

                    if self.obs_ptr == self.numobs {
                        self.obs_ptr = 0;
                        next_state = State::LLI;
                    }
                },
                State::LLI => {
                    let kernel = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)).unwrap();

                    if ascii_len == 0 {
                        kernel.lli_diff.decompress("");
                    } else {
                        kernel.lli_diff.decompress(&ascii[..1]);
                    }

                    next_state = State::SNR;
                },
                State::SNR => {
                    println!("snr {}/{}", self.obs_ptr, self.numobs);

                    let kernel = self.obs_diff.get_mut(&(self.sv, self.obs_ptr)).unwrap();

                    if ascii_len == 0 {
                        kernel.lli_diff.decompress("");
                    } else {
                        kernel.lli_diff.decompress(&ascii[..1]);
                    }

                    self.obs_ptr += 1;

                    if self.obs_ptr == self.numobs {
                        panic!("END OF OBSs");
                        self.obs_ptr = 0;
                        self.sv_ptr += 1;
                        if self.sv_ptr == self.numsat {
                            self.sv_ptr = 0;
                            next_state = State::EpochGathering;
                        } else {
                            next_state = State::Observation;
                        }
                    } else {
                        next_state = State::LLI;
                    }
                },
                _ => {
                    panic!("INVALID!");
                },
            }

            // content is passed on to user
            if !skipped {
                // special cases
                match self.state {
                    State::EpochGathered => {
                        // format and forward
                        let len = self.epoch_descriptor.len();
                        buf[total..total + len].copy_from_slice(&self.epoch_descriptor.as_bytes());
                        buf[total + total + len] = b'\n'; // \n
                        total += len + 1;
                    },
                    State::Observation | State::LLI => {
                        // observations cannot be passed, because we need to decompress the incoming flags
                    },
                    State::SNR => {},
                    _ => {
                        // pass through
                        buf[total..total + ascii_len].copy_from_slice(&bytes[..ascii_len]);
                        buf[total + ascii_len] = b'\n'; // \n
                        total += ascii_len + 1;
                    },
                }
            }

            // roll left
            // TODO : this is time consuming
            //        we should move on to a double pointer scheme ?
            if self.state.line_terminated() {
                roll_start += 1; // discard EOL
            }

            self.buf.copy_within(roll_start.., 0);
            self.avail -= roll_start;

            self.prev_state = self.state;
            self.state = next_state;
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
    /// EOL is used in the decoding process
    const END_OF_LINE_TERMINATOR: u8 = b'\n';
    /// Whitespace char
    const WHITESPACE_BYTE: u8 = b' ';

    /// Special marker to reset decompression kernels
    const ENCODED_WHITESPACE_CHAR: char = '&';
    /// Special marker to reset decompression kernels
    const KERNEL_RESET_MARKER_CHAR: char = '&';
    /// Whitespace char
    const WHITESPACE_CHAR: char = ' ';

    /// Minimal timestamp length in V2 revision
    const V2_TIMESTAMP_SIZE: usize = 26;
    /// Minimal timestamp length in V3 revision
    const V3_TIMESTAMP_SIZE: usize = 28;

    // /// Max. number of [Observable]s described in a single [State::HeaderSpecs] line
    // const NAX_SPECS_PER_LINE :usize = 9;

    /// Locates first non whitespace within slice
    fn find_non_whitespace(slice: &str) -> Option<usize> {
        slice.chars().position(|b| b != Self::WHITESPACE_CHAR)
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
            first_epoch: true,
            last_was_blank: false,
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
    use super::{find, find_end_of_observation, Decompressor};

    use std::{fs::File, io::Read};

    #[test]
    fn test_find_byte() {
        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(find(&slice, 3), Some(3));
        assert_eq!(find(&slice, 0), Some(0));
        assert_eq!(find(&slice, 8), None);
    }

    #[test]
    fn test_find_observation() {
        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7];
        assert_eq!(find_end_of_observation(&slice), None);

        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7, b' ', 9, 10];
        assert_eq!(find_end_of_observation(&slice), Some(8));

        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7, b' ', 9, b' ', 11, 12];
        assert_eq!(find_end_of_observation(&slice), Some(8));

        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7, b' ', b' ', 10, 11];
        assert_eq!(find_end_of_observation(&slice), Some(9));

        let slice = [0_u8, 1, 2, 3, 4, 5, 6, 7, b' ', b' ', 10, 11, 12, 13];
        assert_eq!(find_end_of_observation(&slice), Some(9));
    }

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
