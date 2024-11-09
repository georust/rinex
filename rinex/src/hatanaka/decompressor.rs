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

            // verify that user buffer has enough capacity:
            // we only proceed if we can actually copy the total amount to be produced
            // to simplify internal logic
            if user_len - user_ptr < self.size_to_produce() {
                println!("user_len: not enough capacity");
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

            if let Ok((next, size)) = self.consume_execute_fsm(offset, buf, user_ptr) {
                self.rd_ptr += offset; // advance pointer

                if self.state.eol_terminated() {
                    // consume \n
                    self.rd_ptr += 1;
                }

                self.state = next;
                user_ptr += size;
            } else {
                println!("consume_exec_fsm: error!");
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
            if self.state == State::Observation {
                self.find_next(Self::WHITESPACE_BYTE)
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
                State::Flags => 1 + (self.numobs - 1) * (16 + 1) + 16,
                _ => unreachable!("internal error"),
            }
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

                // copy to user
                user_buf[user_ptr..user_ptr + ascii_len].copy_from_slice(ascii.as_bytes());
                user_buf[user_ptr + ascii_len] = b'\n';
                forwarded += ascii_len + 1;
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
                forwarded += ascii_len + 1;
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
                forwarded += ascii_len + 1;
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
                // copy epoch description to user
                // TODO: need to squeeze clock offset @ appropriate location
                user_buf[user_ptr] = b' ';
                user_buf[user_ptr+1..user_ptr +1 + self.epoch_desc_len]
                    .copy_from_slice(self.epoch_descriptor.as_bytes());
                user_buf[user_ptr+1 + self.epoch_desc_len] = b'\n';
                forwarded += self.epoch_desc_len + 2;

                next_state = State::Observation;
            },

            State::Observation => {
                if ascii_len > 14 {
                    // on trimmed lines with early epoch termination (= no flag update, all remaning obs blanked)
                    // the whitespace search winds up picking up the following line
                    // That we should not process yet
                    //panic!("oops: \"{}\"", ascii);
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

                        if let Ok(val) = ascii[2..].trim().parse::<i64>() {
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
                            "                ".to_string()
                        }
                    } else {
                        if let Ok(val) = ascii.trim().parse::<i64>() {
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

                // publish this paylad & reset for next time
                user_buf[user_ptr + self.pending_copy] = b'\n';
                forwarded += self.pending_copy + 1; // \n

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

        Ok((next_state, forwarded))
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

#[cfg(test)]
mod test {
    use super::Decompressor;
    use std::{fs::File, io::Read};

    #[test]
    fn test_v1_aopr0017d_raw() {
        let fd = File::open("../test_resources/CRNX/V1/aopr0010.17d").unwrap();
        let mut decompressor = Decompressor::<5, File>::new(fd);

        let mut buf = Vec::<u8>::with_capacity(65536);
        let size = decompressor.read_to_end(&mut buf).unwrap();

        assert!(size > 0);

        let string =
            String::from_utf8(buf).expect("decompressed CRINEX does not containt valid utf8");

        for (nth, line) in string.lines().enumerate() {
            match nth+1 {
                1 => {
                    assert_eq!(line, "     2.10           OBSERVATION DATA    G (GPS)             RINEX VERSION / TYPE");
                },
                2 => {
                    assert_eq!(line, "teqc  2002Mar14     Arecibo Observatory 20170102 06:00:02UTCPGM / RUN BY / DATE");
                },
                3 => {
                    assert_eq!(
                        line,
                        "Linux 2.0.36|Pentium II|gcc|Linux|486/DX+                   COMMENT"
                    );
                },
                13 => {
                    assert_eq!(line, "     5    L1    L2    C1    P1    P2                        # / TYPES OF OBSERV");
                },
                14 => {
                    assert_eq!(
                        line,
                        "Version: Version:                                           COMMENT"
                    );
                },
                18 => {
                    assert_eq!(line, "  2017     1     1     0     0    0.0000000     GPS         TIME OF FIRST OBS");
                },
                19 => {
                    assert_eq!(
                        line,
                        "                                                            END OF HEADER"
                    );
                },
                20 => {
                    assert_eq!(
                        line,
                        " 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26"
                    );
                },
                21 => {
                    assert_eq!(line, " -14746974.73049 -11440396.20948  22513484.6374   22513484.7724   22513487.3704 ");
                },
                22 => {
                    assert_eq!(line, " -19651355.72649 -15259372.67949  21319698.6624   21319698.7504   21319703.7964 ");
                },
                23 => {
                    assert_eq!(line, "  -9440000.26548  -7293824.59347  23189944.5874   23189944.9994   23189951.4644 ");
                },
                24 => {
                    assert_eq!(line, " -11141744.16748  -8631423.58147  23553953.9014   23553953.6364   23553960.7164 ");
                },
                25 => {
                    assert_eq!(line, " -21846711.60849 -16970657.69649  20528865.5524   20528865.0214   20528868.5944 ");
                },
                26 => {
                    assert_eq!(line, "  -2919082.75648  -2211037.84947  24165234.9594   24165234.7844   24165241.6424 ");
                },
                27 => {
                    assert_eq!(line, " -20247177.70149 -15753542.44648  21289883.9064   21289883.7434   21289887.2614 ");
                },
                28 => {
                    assert_eq!(line, " -15110614.77049 -11762797.21948  23262395.0794   23262394.3684   23262395.3424 ");
                },
                29 => {
                    assert_eq!(line, " -16331314.56648 -12447068.51348  22920988.2144   22920987.5494   22920990.0634 ");
                },
                30 => {
                    assert_eq!(line, " -15834397.66049 -12290568.98049  21540206.1654   21540206.1564   21540211.9414 ");
                },
                31 => {
                    assert_eq!(line, " 17  1  1  3 33 40.0000000  0  9G30G27G11G16G 8G 7G23G 9G 1   ");
                },
                32 => {
                    assert_eq!(line, "  -4980733.18548  -3805623.87347  24352349.1684   24352347.9244   24352356.1564 ");
                },
                33 => {
                    assert_eq!(line, "  -9710828.79748  -7513506.68548  23211317.1574   23211317.5034   23211324.2834 ");
                },
                34 => {
                    assert_eq!(line, " -26591640.60049 -20663619.71349  20668830.8234   20668830.4204   20668833.2334 ");
                },
                41 => {
                   assert_eq!(line, " 17  1  1  6  9 10.0000000  0 11G30G17G 3G11G19G 8G 7G 6G22G28G 1");
                },
                _ => {},
            }
        }
    }

    // TODO add more V1 tests
    // TODO add V3 tests
    // TODO add test with CLOCK fields
}
