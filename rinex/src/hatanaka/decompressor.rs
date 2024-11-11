//! CRINEX decompression module
use crate::{
    hatanaka::{Error, NumDiff, TextDiff, CRINEX},
    prelude::{Constellation, Observable, Version, SV},
};

use std::{
    collections::HashMap,
    io::{Read, Result},
    str::{from_utf8, FromStr},
};

use num_integer::div_ceil;

#[cfg(feature = "log")]
use log::debug;

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

#[cfg(docsrs)]
use crate::hatanaka::Compressor;

/// [Reader] is a small private wrapper to support
/// both native and Gzip compressed streams
enum Reader<R: Read> {
    Plain(R),
    Gz(GzDecoder<R>),
}

impl<R: Read> Read for Reader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Plain(r) => r.read(buf),
            Self::Gz(r) => r.read(buf),
        }
    }
}

/// [Decompressor] is a structure to decompress CRINEX (compressed compacted RINEX)
/// into readable RINEX. It implements the same core elements as the historical
/// CRX2RNX tool, so is limited to M<=5. If you're an expert and what full control over the decompression
/// algorithm, move to [DecompressorExpert] instead.
///
/// [Decompressor] is efficient and works from any [Read]able interface.
/// It is somewhat flexible but currently, several severe errors will cause it to panic,
/// someof those being:
///  - numsat incorrectly encoded in Epoch description
///  - bad observable specifications
///  - bad constellation specifications.
///
/// In this example, we deploy the [Decompressor] over a local file, but that is just
/// an example.
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
/// let decomp = Decompressor::new(fd);
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
pub type Decompressor<R> = DecompressorExpert<5, R>;

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
    ObservationV2,
    ObservationV3,
    /// Collecting observation separator
    ObservationSeparator,
    /// We wind up here on early line terminations (all blankings)
    EarlyTerminationV2,
    /// We wind up here on early line terminations (all blankings)
    EarlyTerminationV3,
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
    /// - >
    /// - Timestamp: Year uses 4 digits
    /// - Flag
    /// - Numsat
    const MIN_V3_EPOCH_DESCRIPTION_SIZE: usize = 28 + 3 + 3 + 1;

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
    /// Calculates number of bytes this state will forward to user
    fn size_to_produce(&self, v3: bool, numsat: usize, numobs: usize) -> usize {
        if self.file_header() {
            80
        } else {
            match self {
                Self::ClockGathering => {
                    // total epoch description + potential clock description
                    if v3 {
                        Self::MIN_V3_EPOCH_DESCRIPTION_SIZE
                    } else {
                        let mut size = Self::MIN_V2_EPOCH_DESCRIPTION_SIZE;
                        let num_extra = div_ceil(numsat, 12) - 1;
                        size += num_extra * 17; // padding
                        size += numsat * 3; // formatted
                        size
                    }
                },
                Self::ObservationV2 => {
                    let mut size = 1;
                    size += numobs - 1; // separator
                    size += 15 * numobs; // formatted
                    let num_extra = div_ceil(numobs, 5) - 1;
                    size += num_extra * 15; // padding
                    size
                },
                Self::ObservationV3 => {
                    let size = 3; // SVNN
                    size + numobs * 16
                },
                // In the following states, we're about to produce data.
                // We mark the pending size to the user so the user buffer is locked
                // and cannot be modified.
                Self::Flags => {
                    if v3 {
                        Self::ObservationV3.size_to_produce(true, numsat, numobs)
                    } else {
                        Self::ObservationV2.size_to_produce(false, numsat, numobs)
                    }
                },
                Self::EarlyTerminationV2 => {
                    Self::ObservationV2.size_to_produce(false, numsat, numobs)
                },
                Self::EarlyTerminationV3 => {
                    Self::ObservationV3.size_to_produce(true, numsat, numobs)
                },
                _ => 0,
            }
        }
    }
}

/// [DecompressorExpert] gives you full control over the maximal compression ratio.
/// When decoding, we adapt to the compression ratio applied when the stream was encoded.
/// RNX2CRX is historically limited to M<=5 and in practice, seems to limit itself to
/// M=3, while 5 is said to be the optimal. With [DecompressorExpert] you can support
/// any value. Keep in mind that CRINEX is not a lossless compression for signal observations.
/// The higher the compression order, the larger the error over the siganl observations.
pub struct DecompressorExpert<const M: usize, R: Read> {
    /// Whether this is a V3 parser or not
    v3: bool,
    /// Internal [State]. Use this to determine
    /// whether we're still inside the Header section (algorithm is not active),
    /// or inside file body (algorithm is active).
    state: State,
    /// For internal logic: remains true until
    /// first [Epoch] description was decoded.
    first_epoch: bool,
    /// For internal parsing logic
    inside_v3_specs: bool,
    /// CRINEX header that we did identify.
    /// You should wait until State != State::InsideHeader to consider this field.
    pub crinex: CRINEX,
    /// [Read]able interface
    reader: Reader<R>,
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
    obs_ptr: usize, // inside epoch
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

impl<const M: usize, R: Read> Read for DecompressorExpert<M, R> {
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

                // reset pointers for next time
                self.rd_ptr = 0;
                self.wr_ptr = 0;
                return Ok(user_ptr);
            }

            // collect next data of interest
            let offset = self.collect_gather();
            if offset.is_none() {
                // failed to locate interesting content:
                // need to grab new content while preserving pending data
                println!(
                    "pattern not found! rd_ptr={}/wr_ptr={}",
                    self.rd_ptr, self.wr_ptr
                );

                // need to preserve data that is yet to analyze
                // shift left, and make room for new read
                self.buf.copy_within(self.rd_ptr.., 0);
                self.wr_ptr -= self.rd_ptr;
                self.rd_ptr = 0;
                return Ok(user_ptr);
            }

            // internal verification that located content seems OK
            // could be avoided if collect_gather is robust enough ?
            // is this really needed ?
            let offset = offset.unwrap();
            if !self.check_length(offset) {
                println!("check_len: not enough bytes!");
                break;
            }

            // verify user buffer has enough capacity.
            // for each state, we only proceed if user can absorb next amount of data to be produced.
            // This vastly simplifies internal logic.
            if user_ptr
                + self
                    .state
                    .size_to_produce(self.v3, self.numsat, self.numobs)
                >= user_len
            {
                println!("user_len: not enough capacity | user={}", user_ptr);

                // need to preserve data that is yet to analyze
                // shift left, and make room for new read
                self.buf.copy_within(self.rd_ptr.., 0);
                self.wr_ptr -= self.rd_ptr;
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

impl<const M: usize, R: Read> DecompressorExpert<M, R> {
    /// EOL is used in the decoding process
    const EOL_BYTE: u8 = b'\n';
    /// Whitespace char
    const WHITESPACE_BYTE: u8 = b' ';

    /// Minimal timestamp length in V2 revision
    const V2_TIMESTAMP_SIZE: usize = 24;
    const V2_NUMSAT_OFFSET: usize = Self::V2_TIMESTAMP_SIZE + 4;
    const V2_SV_OFFSET: usize = Self::V2_NUMSAT_OFFSET + 3;

    /// Minimal timestamp length in V3 revision
    const V3_TIMESTAMP_SIZE: usize = 26;
    const V3_NUMSAT_OFFSET: usize = Self::V3_TIMESTAMP_SIZE + 1 + 4;
    const V3_SV_OFFSET: usize = Self::V3_NUMSAT_OFFSET + 9;

    // /// Max. number of [Observable]s described in a single [State::HeaderSpecs] line
    // const NAX_SPECS_PER_LINE :usize = 9;

    /// Locates first given caracther
    fn find_next(&self, byte: u8) -> Option<usize> {
        self.buf[self.rd_ptr..].iter().position(|b| *b == byte)
    }

    /// Returns pointer offset to parse this sv
    fn sv_slice_start(v3: bool, sv_index: usize) -> usize {
        let offset = if v3 {
            Self::V3_SV_OFFSET
        } else {
            Self::V2_SV_OFFSET
        };
        offset + sv_index * 3
    }

    /// Returns next [SV]
    fn next_sv(&self) -> Option<SV> {
        let start = Self::sv_slice_start(self.v3, self.sv_ptr);
        let end = (start + 3).min(self.epoch_desc_len);

        // TODO: this might fail on old rinex single constell that ommit the constellation
        if let Ok(sv) = SV::from_str(&self.epoch_descriptor[start..end].trim()) {
            Some(sv)
        } else {
            None
        }
    }

    /// Macro to directly parse numsat from recovered descriptor
    fn epoch_numsat(&self) -> Option<usize> {
        let start = if self.v3 {
            Self::V3_NUMSAT_OFFSET
        } else {
            Self::V2_NUMSAT_OFFSET
        };

        if let Ok(numsat) = &self.epoch_descriptor[start..start + 3].trim().parse::<u8>() {
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
                State::EarlyTerminationV2 => Some(1),
                State::EarlyTerminationV3 => Some(1),
                State::ObservationSeparator => Some(1),
                State::ObservationV2 | State::ObservationV3 => {
                    self.find_next(Self::WHITESPACE_BYTE)
                },
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
            State::ObservationV2 => true,
            State::ObservationV3 => true,
            State::EpochGathering => true,
            State::ClockGathering => true,
            State::EarlyTerminationV2 => true,
            State::EarlyTerminationV3 => true,
            State::ObservationSeparator => true,
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
                    if self.v3 {
                        if self.inside_v3_specs {
                            panic!("bad crinex: incomplete observable specs");
                        } else {
                            next_state = State::EpochGathering;
                        }
                    } else {
                        // Reached _end_of_header_ without observables specs !
                        panic!("bad crinex: no observable specs");
                    }
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
                    }

                    if self.inside_v3_specs {
                        // parse this line
                        let mut start = 0;
                        let mut end = 11;

                        let end_range = (self.obs_ptr + 13).min(self.numobs);
                        for i in self.obs_ptr..end_range {
                            let slice = &ascii[start..end];
                            println!("FOUND: \"{}\"", slice);
                            self.obs_ptr += 1;

                            start = end;
                            end = start + 4;

                            if i == self.numobs - 1 {
                                self.inside_v3_specs = false;
                            }
                        }
                    } else {
                        let constell = Constellation::from_str(ascii[..3].trim())
                            .expect("bad crinex: invalid constellation");

                        let numobs = ascii[3..6]
                            .trim()
                            .parse::<usize>()
                            .expect("bad crinex: invalid number of observable");

                        self.obs_ptr = 0;

                        // parse first line
                        for i in 0..numobs.min(13) {
                            let start = 6 + i * 4;
                            let slice = &ascii[start..start + 5];
                            println!("FOUND: \"{}\"", slice);
                            let obs = Observable::from_str(slice.trim())
                                .expect("bad crinex: invalid observable");

                            if let Some(list) = self.gnss_observables.get_mut(&constell) {
                                list.push(obs);
                            } else {
                                self.gnss_observables.insert(constell, vec![obs]);
                            }

                            self.obs_ptr += 1;
                        }

                        // helpers for multi line parsing
                        if numobs > 13 {
                            self.numobs = numobs;
                            self.inside_v3_specs = true;
                        } else {
                            self.obs_ptr = 0;
                            self.numobs = 0;
                            self.inside_v3_specs = false;
                        }
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
                self.first_epoch = false;

                // grab first specs
                let obs = self
                    .get_observables(&self.sv.constellation)
                    .expect("fail to determine sv definition");

                // move on to next state
                self.numobs = obs.len();
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
                    user_buf[user_ptr + ptr] = b'>'; // special marker
                    ptr += 1;

                    // push timestamp + flag,
                    // and we squeeze SVNN on each observation line to follow
                    user_buf[user_ptr + ptr..user_ptr + ptr + 34].copy_from_slice(&bytes[..34]);
                    ptr += 34;

                    user_buf[user_ptr + ptr] = b'\n';
                    ptr += 1;
                } else {
                    // tedious format
                    user_buf[user_ptr + ptr] = b' '; // single whitespace
                    ptr += 1;

                    // push first (up to 68) line
                    let first_len = self.epoch_desc_len.min(67);

                    user_buf[user_ptr + ptr..user_ptr + ptr + first_len]
                        .copy_from_slice(&bytes[..first_len]);

                    ptr += first_len;

                    // first eol
                    user_buf[user_ptr + ptr] = b'\n';
                    ptr += 1;

                    // if more  than 1 line is required;
                    // append as many, with "standardized" padding
                    let nb_extra_lines = self.epoch_desc_len / 68;

                    for i in 0..nb_extra_lines {
                        // extra padding
                        user_buf[user_ptr + ptr..user_ptr + ptr + 32].copy_from_slice(&[
                            b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                            b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                            b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                        ]);

                        ptr += 32;

                        // copy appropriate slice
                        let start = (i + 1) * 67;
                        let end = (start + 68).min(self.epoch_desc_len);
                        let size = end - start;

                        user_buf[user_ptr + ptr..user_ptr + ptr + size]
                            .copy_from_slice(&bytes[start..end]);

                        ptr += size;

                        // terminate this line
                        user_buf[user_ptr + ptr] = b'\n';
                        ptr += 1;
                    }
                }

                produced += ptr;

                if self.v3 {
                    next_state = State::ObservationV3;
                } else {
                    next_state = State::ObservationV2;
                }
            },

            State::ObservationV2 => {
                let mut early_termination = false;

                if ascii_len > 13 {
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
                        // This case should never happen.
                        // If we wind up here, the collect_gather and relationship with
                        // the consumed size, is not robust enough
                        unreachable!("internal error");
                    }
                }

                // default is BLANK
                let mut formatted = "                ".to_string();

                // Decoding attempt
                if ascii_len > 0 {
                    let mut kernel_reset = false;

                    if ascii_len > 2 {
                        if ascii[1..2].eq("&") {
                            kernel_reset = false;

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
                                formatted = format!("{:14.3}  ", val).to_string();
                            }
                        }
                    }

                    if !kernel_reset {
                        // regular compression
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
                            formatted = format!("{:14.3}  ", val);
                        }
                    }
                }

                // copy to user
                let mut ptr = self.obs_ptr * 16;
                println!("fmt_len={}", formatted.as_bytes().len());

                // push decompressed content
                user_buf[user_ptr + ptr..user_ptr + ptr + 16].copy_from_slice(formatted.as_bytes());
                ptr += 16;
                self.obs_ptr += 1;

                // v2 case: need to wrapp into several lines....
                if !self.v3 {
                    if self.obs_ptr % 5 == 0 {
                        // line wrapping
                        user_buf[user_ptr + ptr] = b'\n';
                        ptr += 1;

                        // special V2 start of line padding ...
                        user_buf[user_ptr + ptr..user_ptr + ptr + 15].copy_from_slice(&[
                            b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',
                            b' ', b' ', b' ',
                        ]);

                        ptr += 15;
                    }
                }

                println!("obs={}/{}", self.obs_ptr, self.numobs);

                // End of this observation: we have three cases
                //  1. weird case where line is early terminated
                //     due to the combination of Blanking (missing signals) and all flags being fully compressed.
                //     In this scenario, we enter special case to push the necessary blanking
                //     and re-use the past flag description (100% compression)
                //  2. this is a regular BLANK, we need to determine whether this terminates
                //     the observation serie or not
                //  3. regular observation: need to progress to the next one
                if early_termination {
                    // Early termination case
                    next_state = State::EarlyTerminationV2;
                } else {
                    if ascii_len == 0 {
                        // BLANKING case
                        if self.obs_ptr == self.numobs {
                            // Last blanking
                            self.obs_ptr = 0;
                            next_state = State::Flags;
                        } else {
                            // regular progression in case of blanking
                            next_state = State::ObservationV2;
                        }
                    } else {
                        // regular progression
                        next_state = State::ObservationSeparator;
                    }
                }
            },

            State::ObservationV3 => {
                let mut early_termination = false;

                if let Some(eol_offset) = ascii.find('\n') {
                    if eol_offset < ascii_len - 1 {
                        // we grabbed part of the following line
                        // this happens in case all data flags remain identical (100% compression factor)
                        // we must postpone part of this buffer
                        ascii_len = eol_offset;
                        consumed = eol_offset;
                        early_termination = true;
                    }
                }

                // default is BLANK
                let mut formatted = "                ".to_string();

                // Decoding attempt
                if ascii_len > 0 {
                    let mut kernel_reset = false;

                    if ascii_len > 2 {
                        if ascii[1..2].eq("&") {
                            kernel_reset = false;

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
                                formatted = format!("{:14.3}  ", val).to_string();
                            }
                        }
                    }

                    if !kernel_reset {
                        // regular compression
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
                            formatted = format!("{:14.3}  ", val);
                        }
                    }
                }

                // copy to user
                let mut ptr = self.obs_ptr * 16;
                println!("formatted={}[{}]", formatted, formatted.as_bytes().len());

                if self.obs_ptr == 0 {
                    // need to squeeze SVNN at beginning of each line
                    let bytes = self.epoch_descriptor.as_bytes();
                    let start = Self::sv_slice_start(self.v3, self.sv_ptr);

                    user_buf[user_ptr + ptr..user_ptr + ptr + 3]
                        .copy_from_slice(&bytes[start..start + 3]);

                    ptr += 3;
                } else {
                    ptr += 3;
                }

                // push decompressed content
                user_buf[user_ptr + ptr..user_ptr + ptr + 16].copy_from_slice(formatted.as_bytes());

                ptr += 16;
                self.obs_ptr += 1;
                println!("obs={}/{}", self.obs_ptr, self.numobs);

                // End of this observation: we have three cases
                //  1. weird case where line is early terminated
                //     due to the combination of Blanking (missing signals) and all flags being fully compressed.
                //     In this scenario, we enter special case to push the necessary blanking
                //     and re-use the past flag description (100% compression)
                //  2. this is a regular BLANK, we need to determine whether this terminates
                //     the observation serie or not
                //  3. regular observation: need to progress to the next one

                if early_termination {
                    // Early termination case
                    next_state = State::EarlyTerminationV3;
                } else {
                    if ascii_len == 0 {
                        // BLANKING case
                        if self.obs_ptr == self.numobs {
                            // Last blanking
                            self.obs_ptr = 0;
                            next_state = State::Flags;
                        } else {
                            // regular progression in case of blanking
                            next_state = State::ObservationV3;
                        }
                    } else {
                        // regular progression
                        next_state = State::ObservationSeparator;
                    }
                }
            },

            State::ObservationSeparator => {
                if self.obs_ptr == self.numobs {
                    self.obs_ptr = 0;
                    next_state = State::Flags;
                } else {
                    if self.v3 {
                        next_state = State::ObservationV3;
                    } else {
                        next_state = State::ObservationV2;
                    }
                }
            },

            State::Flags => {
                // recover flags
                self.flags_descriptor = self.flags_diff.decompress(&ascii).to_string();
                let flags_len = self.flags_descriptor.len();
                println!("RECOVERED: \"{}\"", self.flags_descriptor);

                let flags_bytes = self.flags_descriptor.as_bytes();

                let mut ptr = if self.v3 { 17 } else { 16 };

                // copy all flags to user
                for i in 0..self.numobs {
                    let lli_idx = i * 2;
                    if flags_len > lli_idx {
                        user_buf[user_ptr + ptr] = flags_bytes[lli_idx];
                    }
                    ptr += 1;
                    let snr_idx = lli_idx + 1;
                    if flags_len > snr_idx {
                        user_buf[user_ptr + ptr] = flags_bytes[snr_idx];
                    }
                    ptr += 15;
                }

                ptr -= 15;

                // publish this payload & reset for next time
                user_buf[user_ptr + ptr] = b'\n';

                if self.v3 {
                    produced +=
                        State::ObservationV3.size_to_produce(true, self.numsat, self.numobs);
                } else {
                    produced +=
                        State::ObservationV2.size_to_produce(false, self.numsat, self.numobs);
                }

                self.sv_ptr += 1;
                println!("COMPLETED {}", self.sv);

                if self.sv_ptr == self.numsat {
                    self.sv_ptr = 0;
                    next_state = State::EpochGathering;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next vehicle");
                    if self.v3 {
                        next_state = State::ObservationV3;
                    } else {
                        next_state = State::ObservationV2;
                    }
                }
            },

            State::EarlyTerminationV2 => {
                // Special case where line is abruptly terminated
                // - all remaining observations have been blanked (=missing)
                // - all flags were omitted (= to remain identical 100% compressed)

                // we need to fill the blanks
                let mut ptr = 1 + 16 * self.obs_ptr;

                for _ in self.obs_ptr..self.numobs {
                    println!("BLANK(early) ={}/{}", ptr + 1, self.numobs);
                    let blanking = "                ".to_string();

                    // copy to user;

                    user_buf[user_ptr + ptr..user_ptr + ptr + 16]
                        .copy_from_slice(blanking.as_bytes());

                    ptr += 16;
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
                    }

                    let start = 15 + i * 16;
                    if flags_len > snr_idx {
                        // user_buf[user_ptr + start] = b'y';
                        user_buf[user_ptr + start] = flags_bytes[snr_idx];
                    }
                }

                // publish this payload
                user_buf[user_ptr + ptr] = b'\n';
                produced += 0; // TODO

                // move on to next state
                self.obs_ptr = 0;
                self.sv_ptr += 1;
                println!("COMPLETED {}", self.sv);

                if self.sv_ptr == self.numsat {
                    self.sv_ptr = 0;
                    next_state = State::EpochGathering;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next vehicle");
                    next_state = State::ObservationV2;
                }
            },

            State::EarlyTerminationV3 => {
                // Special case where line is abruptly terminated
                // - all remaining observations have been blanked (=missing)
                // - all flags were omitted (= to remain identical 100% compressed)

                // we need to fill the blanks
                let mut ptr = 3 + 16 * self.obs_ptr;

                for _ in self.obs_ptr..self.numobs {
                    println!("BLANK(early) ={}/{}", ptr + 1, self.numobs);
                    let blanking = "                ".to_string();

                    // copy to user
                    user_buf[user_ptr + ptr..user_ptr + ptr + 16]
                        .copy_from_slice(blanking.as_bytes());

                    ptr += 16;
                }

                // all data flags are maintained
                let flags_len = self.flags_descriptor.len();
                let flags_bytes = self.flags_descriptor.as_bytes();
                println!("RECOVERED: \"{}\"", self.flags_descriptor);

                // copy all flags to user
                let mut ptr = 3 + 16;
                for i in 0..self.numobs {
                    let lli_idx = i * 2;
                    let snr_idx = lli_idx + 1;
                    if flags_len > lli_idx {
                        user_buf[user_ptr + ptr] = b'x';
                        //user_buf[user_ptr + start] = flags_bytes[lli_idx];
                    }
                    if flags_len > snr_idx {
                        user_buf[user_ptr + ptr + 1] = b'y';
                        //user_buf[user_ptr + ptr +1] = flags_bytes[snr_idx];
                    }
                    ptr += 15;
                }

                // publish this payload
                ptr -= 15;
                user_buf[user_ptr + ptr] = b'\n';
                produced += ptr;

                // move on to next state
                self.obs_ptr = 0;
                self.sv_ptr += 1;
                println!("COMPLETED {}", self.sv);

                if self.sv_ptr == self.numsat {
                    self.sv_ptr = 0;
                    next_state = State::EpochGathering;
                } else {
                    self.sv = self.next_sv().expect("failed to determine next vehicle");
                    next_state = State::ObservationV3;
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

    /// Creates a new [Decompressor] to work from [Read]able stream
    pub fn new(reader: R) -> Self {
        Self {
            v3: false,
            wr_ptr: 0,
            rd_ptr: 0,
            numsat: 0,
            sv_ptr: 0,
            numobs: 0,
            obs_ptr: 0,
            buf: [0; 4096],
            eos: false,
            first_epoch: true,
            inside_v3_specs: false,
            sv: Default::default(),
            state: Default::default(),
            crinex: Default::default(),
            reader: Reader::Plain(reader),
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

    /// Creates a new [Decompressor] to work from gzip compressed [Read]able stream
    #[cfg(feature = "flate2")]
    pub fn new_gzip(reader: R) -> Self {
        Self {
            v3: false,
            wr_ptr: 0,
            rd_ptr: 0,
            numsat: 0,
            sv_ptr: 0,
            numobs: 0,
            obs_ptr: 0,
            buf: [0; 4096],
            eos: false,
            first_epoch: true,
            inside_v3_specs: false,
            sv: Default::default(),
            state: Default::default(),
            crinex: Default::default(),
            flags_diff: TextDiff::new(""),
            epoch_diff: TextDiff::new(""),
            constellation: Constellation::Mixed,
            reader: Reader::Gz(GzDecoder::new(reader)),
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

#[cfg(test)]
mod test {
    use super::{Decompressor, State};
    use crate::prelude::SV;
    use std::fs::File;
    use std::str::FromStr;

    #[test]
    fn epoch_size_to_produce_v2() {
        for (numsat, expected) in [
            (
                9,
                " 17  1  1  3 33 40.0000000  0  9G30G27G11G16G 8G 7G23G 9G 1",
            ),
            (
                10,
                " 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26",
            ),
            (
                11,
                " 17  1  1  0  0  0.0000000  0 11G31G27G 3G32G16G 8G14G23G22G26G27",
            ),
            (
                12,
                " 17  1  1  0  0  0.0000000  0 12G31G27G 3G32G16G 8G14G23G22G26G27G28",
            ),
            (
                13,
                " 21 01 01 00 00 00.0000000  0 13G07G08G10G13G15G16G18G20G21G23G26G27
                G29",
            ),
            (
                14,
                " 21 01 01 00 00 00.0000000  0 14G07G08G10G13G15G16G18G20G21G23G26G27
                G29G30",
            ),
            (
                24,
                " 21 12 21  0  0  0.0000000  0 24G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33",
            ),
            (
                25,
                " 21 12 21  0  0  0.0000000  0 25G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33
                S23",
            ),
            (
                26,
                " 21 12 21  0  0  0.0000000  0 26G07G08G10G16G18G21G23G26G32R04R05R10
                R12R19R20R21E04E11E12E19E24E25E31E33
                S23S36",
            ),
        ] {
            let size = State::ClockGathering.size_to_produce(false, numsat, 0);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn data_size_to_produce_v2() {
        for (numobs, expected) in [
            (1, " 110158976.908 8"),
            (2, " 110158976.908 8  85838153.10248"),
            (3, " 110158976.908 8  85838153.10248  20962551.380  "),
            (
                4,
                " 119147697.073 7  92670417.710 7  22249990.480    22249983.480  ",
            ),
            (
                5,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  ",
            ),
            (
                6,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140  ",
            ),
            (
                9,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600  ",
            ),
            (
                10,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600          41.650  ",
            ),
            (
                14,
                "  24017462.340       -3054.209       -2379.903          43.650          41.600  
                25509828.140        2836.327        2210.128          41.600          41.650  
               100106048.706 6  25509827.540        2118.232          39.550  ",
            ),
        ] {
            let size = State::Flags.size_to_produce(false, 0, numobs);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn epoch_size_to_produce_v3() {
        for (numsat, expected) in [
            (18, "> 2022 03 04 00 00  0.0000000  0 18"),
            (22, "> 2022 03 04 00 00  0.0000000  0 22"),
        ] {
            let size = State::ClockGathering.size_to_produce(true, numsat, 0);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn data_size_to_produce_v3() {
        for (numobs, expected) in [
            (1, "G01  20243517.560  "),
            (2, "G03  20619020.680   108353702.79708"),
            (4, "R10  22432243.520   119576492.91607      1307.754          43.250  "),
            (8, "R17  20915624.780   111923741.34508      1970.309          49.000    20915629.120    87051816.58507      1532.457          46.500  "),
        ] {
            let size = State::Flags.size_to_produce(true, 0, numobs);
            assert_eq!(size, expected.len(), "failed for \"{}\"", expected);
        }
    }

    #[test]
    fn v2_sv_slice() {
        let recovered = "21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27G30R01R02R03R08R09R15R16R17R18R19R20";
        for sv_index in 0..24 {
            let start = Decompressor::<File>::sv_slice_start(false, sv_index);
            let slice_str = &recovered[start..start + 3];
            if sv_index == 0 {
                assert_eq!(slice_str, "G07");
            } else if sv_index == 1 {
                assert_eq!(slice_str, "G08");
            }
            let _ = SV::from_str(slice_str.trim()).unwrap();
        }
    }

    #[test]
    fn v2_numsat_slice() {
        let recovered = "21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27G30R01R02R03R08R09R15R16R17R18R19R20";
        let offset = Decompressor::<File>::V2_NUMSAT_OFFSET;
        let numsat_str = &recovered[offset..offset + 3];
        assert_eq!(numsat_str, " 24");
        let numsat = numsat_str.trim().parse::<u64>().unwrap();
        assert_eq!(numsat, 24);
    }

    #[test]
    fn v3_sv_slice() {
        let recovered = " 2020 06 25 00 00 00.0000000  0 43      C05C07C10C12C19C20C23C32C34C37E01E03E05E09E13E15E24E31G02G05G07G08G09G13G15G18G21G27G28G30R01R02R08R09R10R11R12R17R18R19S23S25S36";
        for sv_index in 0..43 {
            let start = Decompressor::<File>::sv_slice_start(true, sv_index);
            let slice_str = &recovered[start..start + 3];
            if sv_index == 0 {
                assert_eq!(slice_str, "C05");
            } else if sv_index == 1 {
                assert_eq!(slice_str, "C07");
            }
            let _ = SV::from_str(slice_str.trim()).unwrap();
        }
    }

    #[test]
    fn v3_numsat_slice() {
        let recovered = " 2020 06 25 00 00 00.0000000  0 43      C05C07C10C12C19C20C23C32C34C37E01E03E05E09E13E15E24E31G02G05G07G08G09G13G15G18G21G27G28G30R01R02R08R09R10R11R12R17R18R19S23S25S36";
        let offset = Decompressor::<File>::V3_NUMSAT_OFFSET;
        let numsat_str = &recovered[offset..offset + 3];
        assert_eq!(numsat_str, " 43");
        let numsat = numsat_str.trim().parse::<u64>().unwrap();
        assert_eq!(numsat, 43);
    }
}
