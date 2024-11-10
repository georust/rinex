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
    /// Expecting RINEX VERSION / TYPE
    VersionType,
    /// Expecting SYS TYPES OF OBS
    HeaderSpecs,
    /// Wait for END OF HEADER
    EndofHeader,
    /// Collecting Epoch description
    EpochGathering,
    /// Collecting clock data
    ClockGathering,
    /// Collecting new observation
    Observation,
    /// Collecting observation separator
    ObservationSeparator,
    /// We wind up here when early line termination (all blankings) is spotted
    ObservationEarlyTermination,
    /// Collecting observation flags
    Flags,
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
        !self.file_header()
    }

    /// Returns true if we're inside the File Header.
    /// Use this to determine it is not safe to grab the [CRINEX] definition yet.
    pub fn file_header(&self) -> bool {
        matches!(
            self,
            Self::Version
                | Self::ProgDate
                | Self::VersionType
                | Self::HeaderSpecs
                | Self::EndofHeader
        )
    }

    /// True if this [State] needs to collect \n terminated.
    /// Other states are either bytewise dependent (fixed length) or other specific byte to locate
    fn eol_terminated(&self) -> bool {
        self.file_header()
            || matches!(
                self,
                Self::EpochGathering | Self::ClockGathering | Self::Flags
            )
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
/// It currently has one limitation: it requires the user buffer (.read(buf)) to
/// be at least 4096 byte deep.
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
    /// Whether this is a V3 parser or not
    v3: bool,
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
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
    rd_ptr: usize,
    wr_ptr: usize,
    eos: bool,
    /// pointers
    numsat: usize, // total
    sv_ptr: usize, // inside epoch
    sv: SV,
    /// pointers
    numobs: usize, // total
    obs_ptr: usize,      // inside epoch
    pending_copy: usize, // pending copy to user
    /// [TextDiff] that works on entire Epoch line
    epoch_diff: TextDiff,
    /// Epoch descriptor, for single allocation
    epoch_descriptor: String,
    epoch_desc_len: usize, // for internal logic
    /// [TextDiff] for observation flags
    flags_diff: TextDiff,
    /// Recovered flags
    flags_descriptor: String,
    /// Clock offset differentiator
    clock_diff: NumDiff<M>,
    /// Observation differentiators
    obs_diff: HashMap<(SV, usize), NumDiff<M>>,
    /// [Constellation] described in Header field
    constellation: Constellation,
    /// [Observable]s specs for each [Constellation]
    gnss_observables: HashMap<Constellation, Vec<Observable>>,
}

impl<const M: usize, R: Read> Read for Decompressor<M, R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut user_ptr = 0;
        let user_len = buf.len();

        // always try to grab new content
        if self.wr_ptr < 4096 {
            // try to fill internal buffer
            let size = self.reader.read(&mut self.buf[self.wr_ptr..])?;
            self.eos = size == 0;
            self.wr_ptr += size;
            println!("new read: {}", self.wr_ptr);

            if size == 0 {
                // did not grab anything new
                if self.rd_ptr == 0 {
                    // no pending analysis: mark EOS
                    return Ok(0);
                }
            }
        }

        // run FSM
        loop {
            if self.rd_ptr >= self.wr_ptr {
                // analyzed everything: need to grab new content
                println!(
                    "consumed everything rd={}|wr={}|user={}",
                    self.rd_ptr, self.wr_ptr, user_ptr
                );
                self.rd_ptr = 0;
                self.wr_ptr = 0;
                return Ok(user_ptr);
            }

            // collect next data of interest
            let offset = self.collect_gather();
            if offset.is_none() {
                // failed to locate interesting content:
                // need to grab new content while preserving pending data
                //let ascii = from_utf8(&self.buf[self.rd_ptr..]).unwrap();
                //println!("pattern not found!rd_ptr={} | \"{}\"", self.rd_ptr, ascii);
                println!(
                    "pattern not found! rd_ptr={}/wr_ptr={}",
                    self.rd_ptr, self.wr_ptr
                );
                self.buf.copy_within(self.rd_ptr.., 0);
                self.wr_ptr -= self.rd_ptr;
                self.rd_ptr = 0;
                return Ok(user_ptr);
            }

            // verify there is actually enough content to proceed
            let offset = offset.unwrap();
            if !self.check_length(offset) {
                println!("check_len: not enough bytes!");
                break;
            }

            // verify user buffer has enough capacity:
            // we only proceed if user can absorb total mount to be produced,
            // to simplify internal logic
            if user_ptr + self.size_to_produce() >= user_len {
                println!("user_len: not enough capacity | user={}", user_ptr);
                self.wr_ptr = 0;
                self.rd_ptr = 0;
                return Ok(user_ptr);
            }

            // #[cfg(feature = "log")]
            println!(
                "[V{}/{}] {:?} wr={}/rd={}/user={}",
                self.crinex.version.major,
                self.constellation,
                self.state,
                self.wr_ptr,
                self.rd_ptr,
                user_ptr,
            );

            if let Ok((next, consumed_size, produced_size)) =
                self.consume_execute_fsm(offset, buf, user_ptr)
            {
                self.state = next;
                self.rd_ptr += consumed_size;
                user_ptr += produced_size;
            } else {
                println!("consume_exec_fsm: error!");
                break;
            }
        }

        // // we're done analyzing: concluding publication to user.
        // // If all vast majority of size available has been analyzed:
        // // we allow for a new read (I/O). It will most likely
        // // split in the middle of a line that we need to preversed for next analysis.
        // if self.rd_ptr > self.wr_ptr - 256 {
        //     self.buf.copy_within(self.rd_ptr.., 0);
        //     self.wr_ptr -= self.rd_ptr; // preserve pending analysis
        //     self.rd_ptr = 0; // restart analysis
        //     println!("leftovers: {}/user={}", self.wr_ptr, user_ptr);
        // }

        if user_ptr == 0 {
            // we have not produced a single byte
            if self.eos {
                // because EOS has been reached
                Ok(0)
            } else {
                // we need other I/O access
                Err(Error::NeedMoreData.to_stdio())
            }
        } else {
            Ok(user_ptr)
        }
    }
}

impl<const M: usize, R: Read> Decompressor<M, R> {
    /// EOL is used in the decoding process
    const EOL_BYTE: u8 = b'\n';
    /// Whitespace char
    const WHITESPACE_BYTE: u8 = b' ';

    /// Minimal timestamp length in V2 revision
    const V2_TIMESTAMP_SIZE: usize = 25;
    /// Minimal timestamp length in V3 revision
    const V3_TIMESTAMP_SIZE: usize = 27;

    // /// Max. number of [Observable]s described in a single [State::HeaderSpecs] line
    // const NAX_SPECS_PER_LINE :usize = 9;

    /// Locates first given caracther
    fn find_next(&self, byte: u8) -> Option<usize> {
        self.buf[self.rd_ptr..].iter().position(|b| *b == byte)
    }

    /// Returns pointer offset to parse this sv
    fn sv_slice_start(v3: bool, sv_index: usize) -> usize {
        let mut offset = 6 + 3 * sv_index;
        if v3 {
            offset += Self::V3_TIMESTAMP_SIZE;
        } else {
            offset += Self::V2_TIMESTAMP_SIZE;
        }
        offset
    }

    /// Returns next [SV]
    fn next_sv(&self) -> Option<SV> {
        let offset = Self::sv_slice_start(self.v3, self.sv_ptr);

        if self.epoch_desc_len < offset + 3 {
            return None;
        }

        // TODO: this might fail on old rinex single constell that ommit the constellation
        if let Ok(sv) = SV::from_str(&self.epoch_descriptor[offset..offset + 3].trim()) {
            Some(sv)
        } else {
            None
        }
    }

    /// Macro to directly parse numsat from recovered descriptor
    fn epoch_numsat(&self) -> Option<usize> {
        let mut offset = 3;
        if self.v3 {
            offset += Self::V3_TIMESTAMP_SIZE;
        } else {
            offset += Self::V2_TIMESTAMP_SIZE;
        }

        if let Ok(numsat) = &self.epoch_descriptor[offset..offset + 3]
            .trim()
            .parse::<u8>()
        {
            Some(*numsat as usize)
        } else {
            None
        }
    }

    /// Collect & gather data we're interested starting at current pointer
    fn collect_gather(&mut self) -> Option<usize> {
        if self.state.eol_terminated() {
            self.find_next(Self::EOL_BYTE)
        } else {
            match self.state {
                State::Observation => self.find_next(Self::WHITESPACE_BYTE),
                State::ObservationEarlyTermination => Some(1),
                State::ObservationSeparator => Some(1),
                _ => unreachable!("internal error"),
            }
        }
    }

    /// Verifies that collected data is actually enough to proceed to actual FSM
    fn check_length(&self, size: usize) -> bool {
        println!("\n{:?}: check len={}", self.state, size);
        match self.state {
            State::Version => size > 60,
            State::ProgDate => size > 60,
            State::EndofHeader => size > 60,
            State::VersionType => size > 60,
            State::HeaderSpecs => size > 60,
            State::Flags => true,
            State::Observation => true,
            State::EpochGathering => true,
            State::ClockGathering => true,
            State::ObservationSeparator => true,
            State::ObservationEarlyTermination => true,
        }
    }

    /// Calculates total pending copy to user size
    fn size_to_produce(&self) -> usize {
        if self.state.file_header() {
            80
        } else {
            match self.state {
                State::EpochGathering => {
                    // we cannot copy the epoch descriptor
                    // until clock data has not been gathered
                    0
                },
                State::ClockGathering => {
                    // total epoch description + potential clock description
                    self.epoch_desc_len
                },
                State::Observation => {
                    // we cannot copy until flags have been recovered
                    0
                },
                State::ObservationSeparator => {
                    // we cannot copy until flags have been recovered
                    0
                },
                State::Flags | State::ObservationEarlyTermination => 1 + 32 * self.numobs,
                _ => unreachable!("internal error"),
            }
        }
    }

    /// Process collected bytes that need to be valid UTF-8.
    /// Returns
    /// - next [State]
    /// - consumed size (rd pointer)
    /// - produced size (wr pointer)
    fn consume_execute_fsm(
        &mut self,
        offset: usize,
        user_buf: &mut [u8],
        user_ptr: usize,
    ) -> Result<(State, usize, usize)> {
        let mut produced = 0;
        let mut next_state = self.state;

        // always interprate new content as ASCII UTF-8
        let ascii = from_utf8(&self.buf[self.rd_ptr..self.rd_ptr + offset])
            .map_err(|_| Error::BadUtf8Data.to_stdio())?
            .trim_end(); // clean up

        // we'll consume this ASCII length (total) in most cases
        // except in rare scenarios, mostly in Observation state
        // where actual read size cannot be predicted ahead of time.
        // For such scenarios, check where this variable is modified.
        // The .max(1) is here to consume at least one byte (always)
        // which is not discarded in empty lines, and this block must pass through the \n termination.
        let mut ascii_len = ascii.len();
        let mut consumed = ascii_len.max(1);

        // #[cfg(feature = "log")]
        println!("ASCII: \"{}\" [{}]", ascii, ascii_len);

        // process according to FSM
        match self.state {
            State::Version => {
                let version = Version::from_str(&ascii[0..20].trim())
                    .map_err(|_| Error::VersionParsing.to_stdio())?;

                self.crinex = self.crinex.with_version(version);
                self.v3 = version.major == 3;
                next_state = State::ProgDate;
            },

            State::ProgDate => {
                self.crinex = self
                    .crinex
                    .with_prog_date(&ascii.trim_end())
                    .map_err(|_| Error::CrinexParsing.to_stdio())?;

                next_state = State::VersionType;
            },

            State::VersionType => {
                if ascii.ends_with("END OF HEADER") {
                    // Reached END_OF_HEADER and HeaderSpecs were not identified.
                    // We would not be able to proceed to decode data.
                    panic!("bad crinex: no observation specs");
                }
                if ascii.contains("TYPES OF OBS") {
                    // Reached next specs without specifying a constellation system !
                    panic!("bad crinex: no constellation specs");
                }
                if ascii.contains("SYS / # / OBS TYPES") {
                    // Reached next specs without specifying a constellation system !
                    panic!("bad crinex: no constellation specs");
                }
                if ascii.contains("RINEX VERSION / TYPE") {
                    self.constellation = Constellation::from_str(ascii[40..60].trim())
                        .expect("bad crinex: invalid constellation");

                    self.numobs = 0;
                    self.obs_ptr = 0;
                    next_state = State::HeaderSpecs;
                }

                // copy to user
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                user_buf[user_ptr + ascii_len] = b'\n';
                produced += ascii_len + 1;
            },

            State::HeaderSpecs => {
                if ascii.ends_with("END OF HEADER") {
                    // Reached next specs without specifying observables !
                    panic!("bad crinex: no observable specs");
                }

                if ascii.contains("TYPES OF OBS") {
                    // V2 parsing
                    if self.v3 {
                        panic!("bad v3 crinex definition");
                    }

                    if self.numobs == 0 {
                        // first encounter
                        if let Ok(numobs) = ascii[..10].trim().parse::<u8>() {
                            self.numobs = numobs as usize;
                        }
                    }

                    let num = ascii_len / 6;
                    for i in 0..num {
                        let start = i * 6;
                        if let Ok(obs) = Observable::from_str(&ascii[start..start + 6]) {
                            println!("found {}", obs);
                            if let Some(specs) = self.gnss_observables.get_mut(&self.constellation)
                            {
                                specs.push(obs);
                            } else {
                                self.gnss_observables.insert(self.constellation, vec![obs]);
                            }
                            self.obs_ptr += 1;
                        }
                    }

                    if self.obs_ptr == self.numobs {
                        self.obs_ptr = 0;
                        self.numobs = 0;
                        next_state = State::EndofHeader;
                    }
                }

                if ascii.ends_with("SYS / # / OBS TYPES") {
                    if !self.v3 {
                        panic!("bad v1 crinex definition");
                    } else {
                        panic!("v3: not yet");
                    }
                }

                // copy to user
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                user_buf[user_ptr + ascii_len] = b'\n';
                produced += ascii_len + 1;
            },

            State::EndofHeader => {
                if ascii.ends_with("END OF HEADER") {
                    // move on to next state: prepare for decoding
                    next_state = State::EpochGathering;
                    self.epoch_desc_len = 0;
                    self.epoch_descriptor.clear();
                }

                // copy to user
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                user_buf[user_ptr + ascii_len] = b'\n';
                produced += ascii_len + 1;
            },

            State::EpochGathering => {
                if ascii.starts_with('&') {
                    self.epoch_desc_len = ascii_len - 1;
                    self.epoch_diff.force_init(&ascii[1..]);
                    self.epoch_descriptor = ascii[1..].to_string();
                } else if ascii.starts_with('>') {
                    self.epoch_desc_len = ascii_len - 1;
                    self.epoch_diff.force_init(&ascii[1..]);
                    self.epoch_descriptor = ascii[1..].to_string();
                } else {
                    if self.first_epoch {
                        panic!("bad crinex: first epoch not correctly marked");
                    }

                    if ascii_len > 1 {
                        self.epoch_descriptor = self.epoch_diff.decompress(&ascii[1..]).to_string();
                    } else {
                        self.epoch_descriptor = self.epoch_diff.decompress("").to_string();
                    }

                    self.epoch_desc_len = self.epoch_descriptor.len();
                    println!("RECOVERED: \"{}\"", self.epoch_descriptor);
                }

                // parsing & verification
                self.numsat = self.epoch_numsat().expect("bad epoch recovered (numsat)");

                // grab first SV
                self.sv = self.next_sv().expect("failed to determine sv definition");

                self.obs_ptr = 0;
                self.pending_copy = 1; // initial whitespace

                // grab first specs
                let obs = self
                    .get_observables(&self.sv.constellation)
                    .expect("fail to determine sv definition");

                self.numobs = obs.len();

                // move on to next state
                self.first_epoch = false;
                next_state = State::ClockGathering;
            },

            State::ClockGathering => {
                // copy & format epoch description to user
                // TODO: this misses the clock offset @ appropriate location

                let mut ptr = 0;
                let bytes = self.epoch_descriptor.as_bytes();

                // format according to standards
                if self.v3 {
                    // easy format
                } else {
                    // tedious format
                    user_buf[user_ptr + ptr] = b' '; // single whitespace
                    ptr += 1;

                    // push first (up to 68) line
                    let first_len = self.epoch_desc_len.min(68);
                    user_buf[user_ptr + ptr..user_ptr + ptr + first_len]
                        .copy_from_slice(&bytes[..first_len]);
                    ptr += first_len;

                    // first eol
                    user_buf[user_ptr + ptr] = b'\n';
                    ptr += 1;

                    // if more  than 1 line is required;
                    // append as many, with "standardized" padding
                    let nb_lines = self.epoch_desc_len / 68;

                    // for i in 1..nb_lines {
                    //     // extra padding
                    //     user_buf[ptr..ptr + 10].copy_from_slice(&[b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ']);
                    //     ptr += 10;

                    //     // copy appropriate slice
                    //     let start = i*68;
                    //     let end = (start + 68).min(self.epoch_desc_len);
                    //     let size = end - start;
                    //     user_buf[ptr..ptr + size].copy_from_slice(&bytes[start..end]);
                    //     ptr += size;

                    //     // terminate this line
                    //     user_buf[ptr] = b'\n';
                    //     ptr += 1;
                    // }
                }

                produced += ptr;
                next_state = State::Observation;
            },

            State::Observation => {
                let mut early_termination = false;

                if ascii_len > 14 {
                    // content looks suspicious:
                    // this happens when all remaining flags are omitted (=BLANKING).
                    // We have two cases
                    if let Some(eol_offset) = ascii.find('\n') {
                        // we grabbed part of the following lines
                        // this happens in case all data flags remain identical (100% compression factor)
                        // we must postpone part of this buffer
                        ascii_len = eol_offset;
                        consumed = eol_offset;
                        early_termination = true;
                    } else {
                        // this case should never happen
                        unreachable!("internal error");
                    }
                }

                let formatted = if ascii_len == 0 {
                    // Missing observation (=BLANK)
                    "                ".to_string()
                } else {
                    // Decoding
                    if ascii[1..2].eq("&") {
                        let order = ascii[0..1]
                            .parse::<usize>()
                            .expect("bad crinex compression level");

                        if let Ok(val) = ascii[2..ascii_len].trim().parse::<i64>() {
                            let val = if let Some(kernel) =
                                self.obs_diff.get_mut(&(self.sv, self.obs_ptr))
                            {
                                kernel.force_init(val, order);
                                val as f64 / 1.0E3
                            } else {
                                let kernel = NumDiff::new(val, order);
                                self.obs_diff.insert((self.sv, self.obs_ptr), kernel);
                                val as f64 / 1.0E3
                            };
                            format!("{:14.3}  ", val).to_string()
                        } else {
                            // bad i64 value
                            println!("BAD i64 \"{}\"", &ascii[2..ascii_len]);
                            "                ".to_string()
                        }
                    } else {
                        if let Ok(val) = ascii[..ascii_len].trim().parse::<i64>() {
                            let val = if let Some(kernel) =
                                self.obs_diff.get_mut(&(self.sv, self.obs_ptr))
                            {
                                kernel.decompress(val) as f64 / 1.0E3
                            } else {
                                let kernel = NumDiff::new(val, M);
                                self.obs_diff.insert((self.sv, self.obs_ptr), kernel);
                                val as f64 / 1.0E3
                            };
                            format!("{:14.3}  ", val)
                        } else {
                            // bad i64 value
                            println!("BAD i64 \"{}\"", &ascii[2..ascii_len]);
                            "                ".to_string()
                        }
                    }
                };

                // copy to user
                let start = self.obs_ptr * 16;

                user_buf[user_ptr + start..user_ptr + start + 16]
                    .copy_from_slice(formatted.as_bytes());

                self.obs_ptr += 1;
                self.pending_copy += 14;
                println!("obs={}/{}", self.obs_ptr, self.numobs);

                // We have three cases
                //  1. weird case where line is early terminated
                //     where we need to provide BLANKING and proceed to flags
                //     with actually re-using the previous flag description (all compressed =100%)
                //  2. this is a regular BLANK, we need to determine whether this terminates
                //     the observation serie or not
                //  3. regular observation
                if early_termination {
                    next_state = State::ObservationEarlyTermination;
                } else {
                    if ascii_len == 0 {
                        // BLANKING
                        if self.obs_ptr == self.numobs {
                            self.obs_ptr = 0;
                            next_state = State::Flags;
                        } else {
                            next_state = State::Observation;
                        }
                    } else {
                        next_state = State::ObservationSeparator;
                    }
                }
            },

            State::ObservationSeparator => {
                if self.obs_ptr == self.numobs {
                    self.obs_ptr = 0;
                    next_state = State::Flags;
                } else {
                    next_state = State::Observation;
                }
            },

            State::Flags => {
                // recover flags
                self.flags_descriptor = self.flags_diff.decompress(&ascii).to_string();
                let flags_len = self.flags_descriptor.len();
                println!("RECOVERED: \"{}\"", self.flags_descriptor);

                let flags_bytes = self.flags_descriptor.as_bytes();

                // copy all flags to user
                for i in 0..self.numobs {
                    let start = 14 + i * 16;
                    let lli_idx = i * 2;
                    let snr_idx = lli_idx + 1;

                    if flags_len > lli_idx {
                        //user_buf[user_ptr + start] = b'x';
                        user_buf[user_ptr + start] = flags_bytes[lli_idx];
                        self.pending_copy += 1;
                    }

                    let start = 15 + i * 16;
                    if flags_len > snr_idx {
                        // user_buf[user_ptr + start] = b'y';
                        user_buf[user_ptr + start] = flags_bytes[snr_idx];
                        self.pending_copy += 1;
                    }
                }

                // publish this payload & reset for next time
                user_buf[user_ptr + self.pending_copy] = b'\n';
                produced += self.pending_copy + 1; // \n

                self.sv_ptr += 1;
                self.pending_copy = 1; // initial whitespace
                println!("COMPLETED {}", self.sv);

                if self.sv_ptr == self.numsat {
                    self.sv_ptr = 0;
                    next_state = State::EpochGathering;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next vehicle");
                    next_state = State::Observation;
                }
            },

            State::ObservationEarlyTermination => {
                // Special case where line is abruptly terminated
                // - all remaining observations have been blanked (=missing)
                // - all flags were omitted (= to remain identical 100% compressed)

                // we need to fill the blanks
                for ptr in self.obs_ptr..self.numobs {
                    println!("BLANK(early) ={}/{}", ptr + 1, self.numobs);
                    let blanking = "                ".to_string();

                    // copy to user
                    let start = ptr * 16;

                    user_buf[user_ptr + start..user_ptr + start + 16]
                        .copy_from_slice(blanking.as_bytes());

                    self.pending_copy += 14;
                }

                // we need to maintain all data flags
                let flags_len = self.flags_descriptor.len();
                let flags_bytes = self.flags_descriptor.as_bytes();
                println!("RECOVERED: \"{}\"", self.flags_descriptor);

                // copy all flags to user
                for i in 0..self.numobs {
                    let start = 14 + i * 16;
                    let lli_idx = i * 2;
                    let snr_idx = lli_idx + 1;

                    if flags_len > lli_idx {
                        //user_buf[user_ptr + start] = b'x';
                        user_buf[user_ptr + start] = flags_bytes[lli_idx];
                        self.pending_copy += 1;
                    }

                    let start = 15 + i * 16;
                    if flags_len > snr_idx {
                        // user_buf[user_ptr + start] = b'y';
                        user_buf[user_ptr + start] = flags_bytes[snr_idx];
                        self.pending_copy += 1;
                    }
                }

                // publish this paylad & reset for next time
                user_buf[user_ptr + self.pending_copy] = b'\n';
                produced += self.pending_copy + 1; // \n

                self.obs_ptr = 0;
                self.sv_ptr += 1;
                self.pending_copy = 1; // initial whitespace
                println!("COMPLETED {}", self.sv);

                if self.sv_ptr == self.numsat {
                    self.sv_ptr = 0;
                    next_state = State::EpochGathering;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next vehicle");
                    next_state = State::Observation;
                }
            },
        }

        if ascii_len > 1 && self.state.eol_terminated() {
            // ascii is trimed to facilitate the parsing & internal analysis
            // but it discards the possible \n termination
            // that this module must pass through
            consumed += 1; // consume \n
        }

        Ok((next_state, consumed, produced))
    }

    /// Creates a new [Decompressor] working from [Read]
    pub fn new(reader: R) -> Self {
        Self {
            v3: false,
            reader,
            wr_ptr: 0,
            rd_ptr: 0,
            numsat: 0,
            sv_ptr: 0,
            numobs: 0,
            obs_ptr: 0,
            pending_copy: 1, // initial whitespace
            buf: [0; 4096],
            eos: false,
            first_epoch: true,
            sv: Default::default(),
            state: Default::default(),
            crinex: Default::default(),
            flags_diff: TextDiff::new(""),
            epoch_diff: TextDiff::new(""),
            constellation: Constellation::Mixed,
            clock_diff: NumDiff::<M>::new(0, M),
            obs_diff: HashMap::with_capacity(8), // cannot be initialized yet
            epoch_desc_len: 0,
            epoch_descriptor: String::with_capacity(256),
            flags_descriptor: String::with_capacity(128),
            gnss_observables: HashMap::with_capacity(4),
        }
    }

    /// Helper to retrieve observable for given system
    fn get_observables(&self, constell: &Constellation) -> Option<&Vec<Observable>> {
        // We use mixed to store a single value for single definitions
        if let Some(mixed) = self.gnss_observables.get(&Constellation::Mixed) {
            Some(mixed)
        } else {
            self.gnss_observables.get(constell)
        }
    }
}
