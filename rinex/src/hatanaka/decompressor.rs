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
    /// Expecting RINEX VERSION / TYPE
    VersionType,
    /// Waiting for observable specifications
    HeaderSpecs,
    /// Waiting for "END OF HEADER" special marker
    EndofHeader,
    EpochGathering,
    ClockGathering,
    Observation,
    ObservationSeparator,
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
    rd_ptr: usize,
    wr_ptr: usize,
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

        if self.wr_ptr < 4096 {
            // try to fill internal buffer
            let size = self.reader.read(&mut self.buf[self.wr_ptr..])?;
            self.eos = size == 0;
            self.wr_ptr += size;
        }

        // Run FSM
        loop {
            // collect next data of interest
            let offset = self.collect_gather();
            if offset.is_none() {
                // fail to locate data of interest
                // need to grab more bytes from interface
                break;
            }

            let offset = offset.unwrap();

            // verify that there is actually enough content to proceed
            if !self.check_length(offset) {
                // TODO trace:
                println!("check_len: not enough bytes!");
                break;
            }

            // #[cfg(feature = "log")]
            println!(
                "[V{}/{}] {:?} wr={}/rd={}/user={}",
                self.crinex.version.major,
                self.constellation,
                self.state,
                self.wr_ptr,
                self.rd_ptr,
                user_len,
            );

            if let Ok((next, size)) = self.consume_execute_fsm(offset, buf, user_ptr, user_len) {
                self.rd_ptr += offset; // advance pointer

                if self.state.eol_terminated() {
                    // consume \n
                    self.rd_ptr += 1;
                }

                self.state = next;
                user_ptr += size;
            } else {
                println!("consum_exec: error!");
                break;
            }
        }

        if user_ptr == 0 {
            if self.eos {
                Ok(0)
            } else {
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
            self.find_next(b'\n')
        } else {
            if self.state == State::Observation {
                self.find_next(b' ')
            } else if self.state == State::ObservationSeparator {
                Some(1)
            } else {
                panic!("collect_gather()");
            }
        }
    }

    /// Verifies that collected data is actually enough to proceed to actual FSM
    fn check_length(&self, size: usize) -> bool {
        println!("\n{:?}: check len={}", self.state, size);
        match self.state {
            State::ProgDate => size > 60,
            State::Version => size > 60,
            State::EndofHeader => size > 60,
            State::VersionType => size > 60,
            State::HeaderSpecs => size > 60,
            State::EpochGathering => true,
            State::ClockGathering => true,
            State::Observation => true,
            State::ObservationSeparator => true,
            State::Flags => true,
        }
    }

    /// Process collected bytes that need to be valid UTF-8.
    /// Returns
    /// - next [State]
    /// - consumed forwarded size (bytewise)
    fn consume_execute_fsm(
        &mut self,
        offset: usize,
        user_buf: &mut [u8],
        user_ptr: usize,
        user_len: usize,
    ) -> Result<(State, usize)> {
        let mut forwarded = 0;
        let mut next_state = self.state;

        // always interprate new content as ASCII UTF-8
        let ascii = from_utf8(&self.buf[self.rd_ptr..self.rd_ptr + offset])
            .map_err(|_| Error::BadUtf8Data.to_stdio())?
            .trim_end(); // clean up

        let ascii_len = ascii.len();

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

                // forward as is
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                forwarded += ascii_len;
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

                // forward as is
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                forwarded += ascii_len;
            },

            State::EndofHeader => {
                if ascii.ends_with("END OF HEADER") {
                    // move on to next state: prepare for decoding
                    next_state = State::EpochGathering;
                    self.epoch_desc_len = 0;
                    self.epoch_descriptor.clear();
                }

                // forward as is
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                forwarded += ascii_len;
            },

            State::EpochGathering => {
                if ascii.starts_with('&') {
                    self.epoch_desc_len = ascii_len;
                    self.epoch_diff.force_init(&ascii);
                    self.epoch_descriptor = ascii.to_string();
                } else if ascii.starts_with('>') {
                    self.epoch_desc_len = ascii_len;
                    self.epoch_diff.force_init(&ascii);
                    self.epoch_descriptor = ascii.to_string();
                } else {
                    if self.first_epoch {
                        panic!("bad crinex: first epoch not correctly marked");
                    }
                    self.epoch_descriptor = self.epoch_diff.decompress(&ascii).to_string();

                    println!("RECOVERED: \"{}\"", self.epoch_descriptor);
                }

                // parsing & verification
                self.numsat = self.epoch_numsat().expect("bad epoch recovered (numsat)");

                // grab first SV
                self.sv = self.next_sv().expect("fail to determine sv definition");

                self.obs_ptr = 0;
                // grab first specs
                let obs = self
                    .get_observables(&self.sv.constellation)
                    .expect("fail to determine sv definition");

                self.numobs = obs.len();

                // TODO forward to user

                // move on to next state
                self.first_epoch = false;
                next_state = State::ClockGathering;
            },

            State::ClockGathering => {
                next_state = State::Observation;
            },

            State::Observation => {
                if ascii.starts_with('&') {
                    let order = ascii[1..2].parse::<usize>().unwrap();
                } else {
                }

                self.obs_ptr += 1;
                println!("obs={}/{}", self.obs_ptr, self.numobs);
                next_state = State::ObservationSeparator;
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
                self.sv_ptr += 1;
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

        Ok((next_state, forwarded))
    }

    /// Creates a new [Decompressor] working from [Read]
    pub fn new(reader: R) -> Self {
        Self {
            v3: false,
            reader,
            wr_ptr: 0,
            rd_ptr: 0,
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
        // lowest level seamless V1 decompression
        // Other testing then move on to tests::reader (BufferedReader) which provides
        //  .lines() Iteratation, which is more suited for indepth testing and easier interfacing.
        let fd = File::open("../test_resources/CRNX/V1/aopr0010.17d").unwrap();

        let mut decompressor = Decompressor::<5, File>::new(fd);

        let mut buf = Vec::<u8>::with_capacity(100000);

        let size = decompressor.read_to_end(&mut buf).unwrap();

        assert!(size > 0);

        let string =
            String::from_utf8(buf).expect("decompressed CRINEX does not containt valid utf8");
    }

    // TODO add more V1 tests
    // TODO add V3 tests
    // TODO add test with CLOCK fields
}
