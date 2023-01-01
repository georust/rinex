//! This library provides a set of tools to parse, analyze
//! and process RINEX files.
//! Refer to README and documentation provided here
//! <https://github.com/gwbres/rinex>
pub mod antex;
pub mod carrier;
pub mod clocks;
pub mod constellation;
pub mod epoch;
pub mod gnss_time;
pub mod hardware;
pub mod hatanaka;
pub mod header;
pub mod ionex;
pub mod merge;
pub mod meteo;
pub mod navigation;
pub mod observation;
pub mod record;
pub mod split;
pub mod sv;
pub mod types;
pub mod version;

mod leap;
mod observable;
mod ground_position;

extern crate num;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate lazy_static;

pub mod reader;
use reader::BufferedReader;
use std::io::Write; //, Read};

pub mod writer;
use writer::BufferedWriter;

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use hifitime::Duration;
use navigation::OrbitItem;
use observable::Observable;
use observation::Crinex;
use version::Version;

// Convenient package to import, that
// comprises all basic and major structures
pub mod prelude {
    pub use crate::constellation::{Augmentation, Constellation};
    pub use crate::epoch::EpochFlag;
    pub use crate::header::Header;
    pub use crate::observable::Observable;
    pub use crate::sv::Sv;
    pub use crate::Rinex;
    pub use hifitime::{Duration, Epoch, TimeScale};
	pub use crate::ground_position::GroundPosition;
}

/// SBAS related package
#[cfg(feature = "sbas")]
pub mod sbas {
    pub use crate::constellation::selection_helper;
}

mod algorithm;

/// Processing package, regroups sampling
/// and file quality analysis.
pub mod processing {
    pub use crate::algorithm::*;
    //pub use differential::*;
    //pub use crate::cs::{CsDetector, CsSelectionMethod, CsStrategy};
}

mod qc;

/// RINEX quality package
pub mod quality {
	pub use crate::qc::{QcOpts, QcReport, HtmlReport};
}

use prelude::*;
use carrier::Carrier;
use gnss_time::TimeScaling;

pub use merge::Merge;
pub use split::Split;

use algorithm::{
	Dcb, Smooth, Decimate,
	Combine, Combination,
	IonoDelayDetector,
};

#[macro_use]
extern crate horrorshow;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_comment {
    ($line: expr) => {
        $line.trim_end().ends_with("COMMENT")
    };
}

/// Macro: used in file creation helper.
/// Returns `str` description, as one letter
/// lowercase, used in RINEX file name to describe
/// the sampling period. RINEX specifications:   
/// “a” = 00:00:00 - 00:59:59   
/// “b” = 01:00:00 - 01:59:59   
/// [...]   
/// "x" = 23:00:00 - 23:59:59
macro_rules! hourly_session {
    ($hour: expr) => {
        if $hour == 23 {
            "x".to_string()
        } else {
            let c: char = ($hour + 97).into();
            String::from(c)
        }
    };
}

#[derive(Clone, Debug, PartialEq)]
/// `Rinex` describes a `RINEX` file.
/// ```
/// use rinex::prelude::*;
/// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
///     .unwrap();
/// // header contains high level information
/// // like file standard revision:
/// assert_eq!(rnx.header.version.major, 2);
/// assert_eq!(rnx.header.version.minor, 11);
/// // general informations
/// assert_eq!(rnx.header.program, "teqc  2019Feb25");
/// assert_eq!(rnx.header.run_by, "Unknown"); // field was empty
/// // File creation date, temporarily stored as a String
/// // value, but that will soon change
/// assert_eq!(rnx.header.date, "20210102 00:01:40UTC");
/// assert_eq!(rnx.header.observer, "H. VAN DER MAREL");
/// assert_eq!(rnx.header.station_id, "13502M004");
///
/// // Constellation describes which kind of vehicules
/// // are to be encountered in the record, or which
/// // GNSS constellation the data will be referred to.
/// // Mixed constellation, means a combination of vehicules or
/// // GNSS constellations is expected
/// assert_eq!(rnx.header.constellation, Some(Constellation::Mixed));
/// // Some information on the hardware being used might be stored
/// println!("{:#?}", rnx.header.rcvr);
/// // WGS84 receiver approximate position
/// println!("{:#?}", rnx.header.coords);
/// // comments encountered in the Header section
/// println!("{:#?}", rnx.header.comments);
/// // sampling interval was set
/// assert_eq!(rnx.header.sampling_interval, Some(Duration::from_seconds(30.0))); // 30s sample rate
/// // record content is RINEX format dependent.
/// // This one is Observation RINEX.
/// // Refer to [record::Record] definitions, to understand
/// // how to browse all RINEX records.
/// let record = rnx.record.as_obs()
///     .unwrap();
/// for (epoch, (clk_offset, observations)) in record {
///     // Do something
/// }
/// // comments encountered in file body
/// // are currently stored like this and indexed by epoch of "appearance"
/// // they are currently not really exploited
/// for (epoch, comment) in rnx.comments {
///     println!("{:?}: \"{:?}\"", epoch, comment);
/// }
/// ```
pub struct Rinex {
    /// `header` field contains general information
    pub header: Header,
    /// `comments` : list of extra readable information,   
    /// found in `record` section exclusively.    
    /// Comments extracted from `header` sections are exposed in `header.comments`
    pub comments: record::Comments,
    /// `record` contains `RINEX` file body
    /// and is type and constellation dependent
    pub record: record::Record,
}

impl Default for Rinex {
    /// Builds a default `RINEX`
    fn default() -> Rinex {
        Rinex {
            header: Header::default(),
            comments: record::Comments::new(),
            record: record::Record::default(),
        }
    }
}

#[derive(Error, Debug)]
/// `RINEX` Parsing related errors
pub enum Error {
    #[error("header parsing error")]
    HeaderError(#[from] header::Error),
    #[error("record parsing error")]
    RecordError(#[from] record::Error),
    #[error("file i/o error")]
    IoError(#[from] std::io::Error),
}

impl Rinex {
    /// Builds a new `RINEX` struct from given header & body sections
    pub fn new(header: Header, record: record::Record) -> Rinex {
        Rinex {
            header,
            record,
            comments: record::Comments::new(),
        }
    }

    /// Returns a copy of self with given header attributes
    pub fn with_header(&self, header: Header) -> Self {
        Rinex {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
        }
    }

    /// Replaces header section
    pub fn replace_header(&mut self, header: Header) {
        self.header = header.clone();
    }

    /// Returns a copy of self with given internal record
    pub fn with_record(&self, record: record::Record) -> Self {
        Rinex {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record: record,
        }
    }

    /// Replaces internal record
    pub fn replace_record(&mut self, record: record::Record) {
        self.record = record.clone();
    }

    /// Converts self to CRINEX (compressed RINEX) format.
    /// If current revision is < 3 then file gets converted to CRINEX1
    /// format, otherwise, modern Observations are converted to CRINEX3.
    /// This has no effect if self is not an Observation RINEX.
    pub fn rnx2crnx(&mut self) {
        if self.is_observation_rinex() {
            let mut crinex = Crinex::default();
            crinex.version.major = match self.header.version.major {
                1 | 2 => 1,
                _ => 3,
            };
            self.header = self.header.with_crinex(crinex);
        }
    }

    /// Converts self to CRINEX1 compressed format,
    /// whatever the RINEX revision might be.
    /// This can be used to "force" compression of a RINEX1 into CRINEX3
    pub fn rnx2crnx1(&mut self) {
        if self.is_observation_rinex() {
            self.header = self.header.with_crinex(Crinex {
                version: Version { major: 1, minor: 0 },
                date: epoch::now(),
                prog: format!("rust-rinex-{}", env!("CARGO_PKG_VERSION")),
            });
        }
    }

    /// Converts self to CRINEX3 compressed format,
    /// whatever the RINEX revision might be.
    /// This can be used to "force" compression of a RINEX1 into CRINEX3
    pub fn rnx2crnx3(&mut self) {
        if self.is_observation_rinex() {
            self.header = self.header.with_crinex(Crinex {
                date: epoch::now(),
                version: Version { major: 3, minor: 0 },
                prog: "rust-crinex".to_string(),
            });
        }
    }

    /// Returns timescale used in this RINEX
    pub fn timescale(&self) -> Option<TimeScale> {
        /*
         * all epochs share the same timescale,
         * by construction & definition.
         * No need to test other epochs */
        if let Some(e) = self.epochs().get(0) {
            Some(e.time_scale)
        } else {
            None
        }
    }

    /// Converts a CRINEX (compressed RINEX) into readable RINEX.
    /// This has no effect if self is not an Observation RINEX.
    pub fn crnx2rnx(&mut self) {
        if self.is_observation_rinex() {
            let params = self.header.obs.as_ref().unwrap();
            self.header = self
                .header
                .with_observation_fields(observation::HeaderFields {
                    crinex: None,
                    codes: params.codes.clone(),
                    clock_offset_applied: params.clock_offset_applied,
                    dcb_compensations: params.dcb_compensations.clone(),
                    scalings: params.scalings.clone(),
                });
        }
    }

    /// IONEX specific filename convention
    fn ionex_filename(&self) -> String {
        let mut ret: String = "ccc".to_string(); // 3 figue Analysis center
        ret.push_str("e"); // extension or region code "G" for global ionosphere maps
        ret.push_str("ddd"); // day of the year of first record
        ret.push_str("h"); // file sequence number (1,2,...) or hour (A, B.., Z) within day
        ret.push_str("yy"); // 2 digit year
        ret.push_str("I"); // ionex
                           //ret.to_uppercase(); //TODO
        ret
    }

    /// Returns filename that would respect naming conventions,
    /// based on self attributes
    pub fn filename(&self) -> String {
        if self.is_ionex() {
            return self.ionex_filename();
        }
        let header = &self.header;
        let rtype = header.rinex_type;
        let nnnn = header.station.as_str()[0..4].to_lowercase();
        //TODO:
        //self.header.date should be a datetime object
        //but it is complex to parse..
        let ddd = String::from("DDD");
        let epoch: Epoch = match rtype {
            types::Type::ObservationData
            | types::Type::NavigationData
            | types::Type::MeteoData
            | types::Type::ClockData => self.epochs()[0],
            _ => todo!(), // other files require a dedicated procedure
        };
        if header.version.major < 3 {
            //TODO
            let (_, _, _, h, _, _, _) = epoch.to_gregorian_utc();
            let s = hourly_session!(h);
            let yy = "YY";
            //let yy = format!("{:02}", epoch.date.year());
            let t: String = match rtype {
                types::Type::ObservationData => {
                    if header.is_crinex() {
                        String::from("d")
                    } else {
                        String::from("o")
                    }
                },
                types::Type::NavigationData => {
                    if let Some(c) = header.constellation {
                        if c == Constellation::Glonass {
                            String::from("g")
                        } else {
                            String::from("n")
                        }
                    } else {
                        String::from("x")
                    }
                },
                types::Type::MeteoData => String::from("m"),
                _ => todo!(),
            };
            format!("{}{}{}.{}{}", nnnn, ddd, s, yy, t)
        } else {
            let m = String::from("0");
            let r = String::from("0");
            //TODO: 3 letter contry code, example: "GBR"
            let ccc = String::from("CCC");
            //TODO: data source
            // R: Receiver (hw)
            // S: Stream
            // U: Unknown
            let s = String::from("R");
            let yyyy = "YYYY"; //TODO
            let hh = "HH"; //TODO
            let mm = "MM"; //TODO
                           //let yyyy = format!("{:04}", epoch.date.year());
                           //let hh = format!("{:02}", epoch.date.hour());
                           //let mm = format!("{:02}", epoch.date.minute());
            let pp = String::from("00"); //TODO 02d file period, interval ?
            let up = String::from("H"); //TODO: file period unit
            let ff = String::from("00"); //TODO: 02d observation frequency 02d
                                         //TODO
                                         //Units of frequency FF. “C” = 100Hz; “Z” = Hz; “S” = sec; “M” = min;
                                         //“H” = hour; “D” = day; “U” = unspecified
                                         //NB - _FFU is omitted for files containing navigation data
            let uf = String::from("Z");
            let c: String = match header.constellation {
                Some(c) => c.to_1_letter_code().to_uppercase(),
                _ => String::from("X"),
            };
            let t: String = match rtype {
                types::Type::ObservationData => String::from("O"),
                types::Type::NavigationData => String::from("N"),
                types::Type::MeteoData => String::from("M"),
                types::Type::ClockData => todo!(),
                types::Type::AntennaData => todo!(),
                types::Type::IonosphereMaps => todo!(),
            };
            let fmt = match header.is_crinex() {
                true => String::from("crx"),
                false => String::from("rnx"),
            };
            format!(
                "{}{}{}{}_{}_{}{}{}{}_{}{}_{}{}_{}{}.{}",
                nnnn, m, r, ccc, s, yyyy, ddd, hh, mm, pp, up, ff, uf, c, t, fmt
            )
        }
    }

    /// Builds a `RINEX` from given file.
    /// Header section must respect labelization standards,
    /// some are mandatory.   
    /// Parses record (file body) for supported `RINEX` types.
    pub fn from_file(path: &str) -> Result<Rinex, Error> {
        /* This will be required if we have make the BufferedReader Hatanaka compliant
        // Grab first 80 bytes to fully determine the BufferedReader attributes.
        // We use the `BufferedReader` wrapper for efficient file browsing (.lines())
        // and builtin CRINEX decompression
        let mut reader = BufferedReader::new(path)?;
        let mut buffer = [0; 80]; // 1st line mandatory size
        let mut line = String::new(); // first line
        if let Ok(n) = reader.read(&mut buffer[..]) {
            if n < 80 {
                panic!("corrupt header 1st line")
            }
            if let Ok(s) = String::from_utf8(buffer.to_vec()) {
                line = s.clone()
            } else {
                panic!("header 1st line is not valid Utf8 encoding")
            }
        }*/

        /*
         *      deflate (.gzip) fd pointer does not work / is not fully supported
         *      at the moment. Let's recreate a new object, it's a little bit
         *      silly, because we actually analyze the 1st line twice,
         *      but Header builder already deduces several things from this line.

                reader.seek(SeekFrom::Start(0))
                    .unwrap();
        */
        // create buffered reader
        let mut reader = BufferedReader::new(path)?;
        // --> parse header fields
        let mut header = Header::new(&mut reader).unwrap();
        // --> parse record (file body)
        //     we also grab encountered comments,
        //     they might serve some fileops like `splice` / `merge`
        let (record, comments) = record::parse_record(&mut reader, &mut header).unwrap();
        Ok(Rinex {
            header,
            record,
            comments,
        })
    }

    /// Returns true if this is an ATX RINEX
    pub fn is_antex_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::AntennaData
    }

    /// Returns true if this is a CLOCK RINEX
    pub fn is_clocks_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::ClockData
    }

    /// Returns true if this is an IONEX file
    pub fn is_ionex(&self) -> bool {
        self.header.rinex_type == types::Type::IonosphereMaps
    }

    /// Returns true if self is a 3D IONEX
    pub fn is_ionex_3d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 3
        } else {
            false
        }
    }

    /// Returns true if self is a 2D IONEX,
    /// ie., fixed altitude mode
    pub fn is_ionex_2d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 2
        } else {
            false
        }
    }

    /// Converts 2D Ionex to 3D ionex by
    /// providing some height maps.
    pub fn with_height_maps(&self, height: BTreeMap<Epoch, ionex::Map>) -> Self {
        let mut s = self.clone();
        s.to_ionex_3d(height);
        s
    }

    /// Add RMS maps to self, for epochs
    /// where such map was not previously provided
    pub fn with_rms_maps(&self, rms: BTreeMap<Epoch, ionex::Map>) -> Self {
        let mut s = self.clone();
        if let Some(r) = s.record.as_mut_ionex() {
            for (e, (_, rms_map, _)) in r.iter_mut() {
                if let Some(m) = rms.get(e) {
                    *rms_map = Some(m.to_vec());
                }
            }
        }
        s
    }

    /// Provide Height maps for epochs where such map was not previously provided
    pub fn to_ionex_3d(&mut self, height: BTreeMap<Epoch, ionex::Map>) {
        if let Some(ionex) = self.header.ionex.as_mut() {
            ionex.map_dimension = 3;
        }
        if let Some(r) = self.record.as_mut_ionex() {
            for (e, (_, _, map_h)) in r.iter_mut() {
                if let Some(m) = height.get(e) {
                    *map_h = Some(m.to_vec());
                }
            }
        }
    }

    /// Returns ionex map borders, as North Eastern
    /// and South Western latitude longitude coordinates,
    /// expressed in ddeg°
    pub fn ionex_map_borders(&self) -> Option<((f64, f64), (f64, f64))> {
        if let Some(params) = &self.header.ionex {
            Some((
                (params.grid.latitude.start, params.grid.longitude.start),
                (params.grid.latitude.end, params.grid.longitude.end),
            ))
        } else {
            None
        }
    }

    /// Returns true if this is a METEO RINEX
    pub fn is_meteo_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::MeteoData
    }

    /// Retruns true if this is a NAV RINEX
    pub fn is_navigation_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::NavigationData
    }

    /// Retruns true if this is an OBS RINEX
    pub fn is_observation_rinex(&self) -> bool {
        self.header.rinex_type == types::Type::ObservationData
    }

    /// Returns `epoch` of first observation
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epochs().get(0).copied()
    }

    /// Returns `epoch` of last observation
    pub fn last_epoch(&self) -> Option<Epoch> {
        let epochs = self.epochs();
        epochs.get(epochs.len() - 1).copied()
    }

    /// Returns sampling interval of this record
    /// if such information is contained in file header.
    ///
    /// Example:
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
    /// assert_eq!(rnx.sampling_interval(), Some(Duration::from_seconds(30.0)));
    /// ```
    pub fn sampling_interval(&self) -> Option<Duration> {
        if let Some(interval) = self.header.sampling_interval {
            Some(interval)
        } else {
            if let Some(dominant) = self
                .epoch_intervals()
                .into_iter()
                .max_by(|(_, x_pop), (_, y_pop)| x_pop.cmp(y_pop))
            {
                Some(dominant.0)
            } else {
                None
            }
        }
    }

    /// Epoch Interval Histogram analysis.
    /// Non steady sample rates are present in IONEX but in practice
    /// might be encountered in other RINEX formats too.
    /// ```
    /// use rinex::prelude::*;
    /// use std::collections::HashMap;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// let histogram: HashMap<_, _> = [
    ///     (Duration::from_seconds(15.0*60.0), 1),
    ///     (Duration::from_seconds(4.0*3600.0 + 45.0*60.0), 2),
    ///     (Duration::from_seconds(25.0*60.0), 1),
    ///     (Duration::from_seconds(5.0*3600.0 + 30.0*60.0), 1),
    /// ].into_iter()
    ///     .collect();
    /// assert_eq!(rinex.epoch_intervals(), histogram);
    /// ```
    pub fn epoch_intervals(&self) -> HashMap<Duration, u32> {
        let mut histogram: HashMap<Duration, u32> = HashMap::new();
        let epochs = self.epochs();
        let mut prev_date = epochs[0];
        for epoch in epochs.iter().skip(1) {
            let dt = *epoch - prev_date;
            if let Some(counter) = histogram.get_mut(&dt) {
                *counter += 1;
            } else {
                histogram.insert(dt, 1);
            }
            prev_date = *epoch;
        }
        histogram
    }

    /// Returns a list of epochs where unexpected data gap happend.
    /// Data gap is determined by comparing |e(k)-e(k-1)| ie., successive epoch intervals,
    /// to [Rinex::sampling_interval].
    pub fn data_gaps(&self) -> Vec<(Epoch, Duration)> {
        if let Some(interval) = self.sampling_interval() {
            let epochs = self.epochs();
            let mut prev = epochs[0];
            epochs
                .iter()
                .skip(1)
                .filter_map(|e| {
                    if *e - prev > interval {
                        let pprev = prev;
                        prev = *e;
                        Some((pprev, *e - pprev))
                    } else {
                        prev = *e;
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Returns list of epoch where an anomaly is reported by the receiver
    pub fn observation_epoch_anomalies(&self) -> Vec<Epoch> {
        let mut ret: Vec<Epoch> = Vec::new();
        if let Some(r) = self.record.as_obs() {
            for ((epoch, flag), _) in r {
                if !flag.is_ok() {
                    ret.push(*epoch);
                }
            }
        }
        ret
    }

    /// Returns `true` if self is a `merged` RINEX file,   
    /// meaning, this file is the combination of two RINEX files merged together.  
    /// This is determined by the presence of a custom yet somewhat standardized `FILE MERGE` comments
    pub fn is_merged(&self) -> bool {
        for (_, content) in self.comments.iter() {
            for c in content {
                if c.contains("FILE MERGE") {
                    return true;
                }
            }
        }
        false
    }

    /// Returns list of epochs contained in self.
    /// Faillible! if self is not iterated by `Epoch`.
    pub fn epochs(&self) -> Vec<Epoch> {
        if let Some(r) = self.record.as_obs() {
            r.iter().map(|((k, _), _)| *k).collect()
        } else if let Some(r) = self.record.as_nav() {
            r.iter().map(|(k, _)| *k).collect()
        } else if let Some(r) = self.record.as_meteo() {
            r.iter().map(|(k, _)| *k).collect()
        } else if let Some(r) = self.record.as_clock() {
            r.iter().map(|(k, _)| *k).collect()
        } else if let Some(r) = self.record.as_ionex() {
            r.iter().map(|(k, _)| *k).collect()
        } else {
            panic!(
                "cannot get an epoch iterator for \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }

    /// Returns epochs where a loss of lock event happened.
    /// This is only relevant on Observation RINEX
    pub fn observation_epoch_lock_loss(&self) -> Vec<Epoch> {
        self.observation_lli_and_mask(observation::LliFlags::LOCK_LOSS)
            .epochs()
    }

    /// Removes all observations where lock condition was lost
    pub fn observation_lock_loss_filter_mut(&mut self) {
        self.observation_lli_and_mask_mut(observation::LliFlags::LOCK_LOSS)
    }

    /// List all constellations contained in record
    pub fn list_constellations(&self) -> Vec<Constellation> {
        let mut ret: Vec<Constellation> = Vec::new();
        match self.header.constellation {
            Some(Constellation::Mixed) => {
                if let Some(r) = self.record.as_obs() {
                    for (_e, (_clk, vehicules)) in r {
                        for (sv, _) in vehicules {
                            if !ret.contains(&sv.constellation) {
                                ret.push(sv.constellation.clone());
                            }
                        }
                    }
                } else if let Some(r) = self.record.as_nav() {
                    for (_, classes) in r {
                        for (class, frames) in classes {
                            if *class == navigation::FrameClass::Ephemeris {
                                for frame in frames {
                                    let (_, sv, _) = frame.as_eph().unwrap();
                                    if !ret.contains(&sv.constellation) {
                                        ret.push(sv.constellation.clone());
                                    }
                                }
                            }
                        }
                    }
                } else if let Some(r) = self.record.as_clock() {
                    for (_, types) in r {
                        for (_, systems) in types {
                            for (system, _) in systems {
                                if let Some(sv) = system.as_sv() {
                                    if !ret.contains(&sv.constellation) {
                                        ret.push(sv.constellation.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Some(c) => ret.push(c),
            None => {},
        }
        ret
    }

    /// Returns list of vehicules per constellation and on an epoch basis
    /// that are closest to Zenith. This is basically a max() operation
    /// on the elevation angle, per epoch and constellation.
    /// This can only be computed on Navigation ephemeris.
    pub fn space_vehicules_best_elevation_angle(&self) -> BTreeMap<Epoch, Vec<Sv>> {
        let mut ret: BTreeMap<Epoch, Vec<Sv>> = BTreeMap::new();
        if let Some(record) = self.record.as_nav() {
            for (e, classes) in record.iter() {
                let mut work: BTreeMap<Constellation, (f64, Sv)> = BTreeMap::new();
                for (class, frames) in classes.iter() {
                    if *class == navigation::FrameClass::Ephemeris {
                        for frame in frames.iter() {
                            let (_, sv, ephemeris) = frame.as_eph().unwrap();
                            let orbits = &ephemeris.orbits;
                            if let Some(elev) = orbits.get("e") {
                                // got an elevation angle
                                let elev = elev.as_f64().unwrap();
                                if let Some((angle, v)) = work.get_mut(&sv.constellation) {
                                    // already have data for this constellation
                                    // how does this compare ?
                                    if *angle < elev {
                                        *angle = elev; // overwrite
                                        *v = sv.clone(); // overwrite
                                    }
                                } else {
                                    // new entry
                                    work.insert(sv.constellation.clone(), (elev, sv.clone()));
                                }
                            }
                        }
                    }
                }
                // build result for this epoch
                let mut inner: Vec<Sv> = Vec::new();
                for (_, (_, sv)) in work.iter() {
                    inner.push(*sv);
                }
                ret.insert(*e, inner.clone());
            }
        }
        ret
    }

    pub fn retain_best_elevation_angles_mut(&mut self) {
        let best_vehicules = self.space_vehicules_best_elevation_angle();
        if let Some(record) = self.record.as_mut_nav() {
            record.retain(|e, classes| {
                let best = best_vehicules.get(e).unwrap();
                classes.retain(|class, frames| {
                    if *class == navigation::FrameClass::Ephemeris {
                        frames.retain(|fr| {
                            let (_, sv, _) = fr.as_eph().unwrap();
                            best.contains(sv)
                        });
                        frames.len() > 0
                    } else {
                        false
                    }
                });
                classes.len() > 0
            });
        }
    }

    /// Returns receiver clock offset, for all epoch that came with such information.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let clock_offsets = rnx.observation_clock_offsets();
    /// for ((epoch, flag), clk_offset) in clock_offsets {
    ///     // epoch: [hifitime::Epoch]
    ///     // flag: [epoch::EpochFlag]
    ///     // clk_offset: receiver clock offset [s]
    /// }
    /// ```
    pub fn observation_clock_offsets(&self) -> BTreeMap<(Epoch, EpochFlag), f64> {
        let mut map: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
        if let Some(r) = self.record.as_obs() {
            for (epoch, (clk, _)) in r.iter() {
                if let Some(clk) = clk {
                    map.insert(*epoch, *clk);
                }
            }
        }
        map
    }

    /// Extracts distant clock offsets
    /// (also refered to as "clock biases") in [s],
    /// on an epoch basis and per space vehicule,
    /// from this Navigation record.
    ///
    /// Example:
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// // Retain G07 + G08 vehicules
    /// // to perform further calculations on these vehicules data (GPS + Svnn filter)
    /// let filter = vec![
    ///     Sv {
    ///         constellation: Constellation::GPS,
    ///         prn: 7,
    ///     },
    ///     Sv {
    ///         constellation: Constellation::GPS,
    ///         prn: 8,
    ///     },
    /// ];
    /// rinex.retain_space_vehicule_mut(filter.clone());
    /// let clk_offsets = rinex.space_vehicules_clock_offset();
    /// for (epoch, sv) in clk_offsets {
    ///     for (sv, offset) in sv {
    ///         // sv: space vehicule,
    ///         // offset: f64 [s]
    ///     }
    /// }
    /// ```
    pub fn space_vehicules_clock_offset(&self) -> BTreeMap<Epoch, BTreeMap<Sv, f64>> {
        let mut results: BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
        if let Some(record) = self.record.as_nav() {
            for (e, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        let mut map: BTreeMap<Sv, f64> = BTreeMap::new();
                        for frame in frames.iter() {
                            let (_, sv, ephemeris) = frame.as_eph().unwrap();
                            map.insert(*sv, ephemeris.clock_bias);
                        }
                        if map.len() > 0 {
                            results.insert(*e, map);
                        }
                    }
                }
            }
        }
        results
    }

    /// Extracts distant clock (offset[s], drift [s.s⁻¹], drift rate [s.s⁻²]) triplet,
    /// on an epoch basis and per space vehicule,
    /// from all Ephemeris contained in this Navigation record.
    /// This does not produce anything if self is not a NAV RINEX
    /// or if this NAV RINEX does not contain any Ephemeris frames.
    /// Use this to process [pseudo_range_to_distance]
    ///
    /// Example:
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// // Retain G07 + G08 vehicules
    /// // to perform further calculations on these vehicules data (GPS + Svnn filter)
    /// let filter = vec![
    ///     Sv {
    ///         constellation: Constellation::GPS,
    ///         prn: 7,
    ///     },
    ///     Sv {
    ///         constellation: Constellation::GPS,
    ///         prn: 8,
    ///     },
    /// ];
    /// rinex.retain_space_vehicule_mut(filter.clone());
    /// let biases = rinex.navigation_clock_biases();
    /// for (epoch, sv) in biases {
    ///     for (sv, (offset, dr, drr)) in sv {
    ///         // sv: space vehicule
    ///         // offset [s]
    ///         // dr: clock drift [s.s⁻¹]
    ///         // drr: clock drift rate [s.s⁻²]
    ///     }
    /// }
    /// ```
    pub fn navigation_clock_biases(&self) -> BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64)>> {
        let mut results: BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64)>> = BTreeMap::new();
        if let Some(r) = self.record.as_nav() {
            for (e, classes) in r {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        let mut map: BTreeMap<Sv, (f64, f64, f64)> = BTreeMap::new();
                        for frame in frames {
                            let (_, sv, ephemeris) = frame.as_eph().unwrap();
                            map.insert(
                                *sv,
                                (
                                    ephemeris.clock_bias,
                                    ephemeris.clock_drift,
                                    ephemeris.clock_drift_rate,
                                ),
                            );
                        }
                        if map.len() > 0 {
                            results.insert(*e, map);
                        }
                    }
                }
            }
        }
        results
    }

    /// Returns list of observables, in the form
    /// of standardized 3 letter codes, that can be found in this record.
    /// This does not produce anything in case of ATX and IONEX records.
    /// In case of NAV record:
    ///    - Ephemeris frames: returns list of Orbits identifier,
    ///    example: "iode", "cus"..
    ///    - System Time Offset frames: list of Time Systems ("GAUT", "GAGP"..)
    ///    - Ionospheric Models: does not apply
    pub fn observables(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        if let Some(obs) = &self.header.obs {
            for (constell, codes) in obs.codes.iter() {
                for code in codes {
                    result.push(format!(
                        "{}:{}",
                        constell.to_3_letter_code(),
                        code.to_string()
                    ))
                }
            }
        } else if let Some(obs) = &self.header.meteo {
            for code in obs.codes.iter() {
                result.push(code.to_string())
            }
        } else if let Some(obs) = &self.header.clocks {
            for code in obs.codes.iter() {
                result.push(code.to_string())
            }
        } else if self.is_navigation_rinex() {
            let record = self.record.as_nav().unwrap();
            for (_, classes) in record.iter() {
                for (class, frames) in classes.iter() {
                    if *class == navigation::FrameClass::Ephemeris {
                        for frame in frames.iter() {
                            let (_, _, ephemeris) = frame.as_eph().unwrap();
                            let orbits = &ephemeris.orbits;
                            for key in orbits.keys() {
                                if !result.contains(key) {
                                    result.push(key.to_string())
                                }
                            }
                        }
                    } else if *class == navigation::FrameClass::SystemTimeOffset {
                        for frame in frames.iter() {
                            let (_, _, sto) = frame.as_sto().unwrap();
                            if !result.contains(&sto.system.to_string()) {
                                result.push(sto.system.clone())
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// Returns list of space vehicules encountered per epoch
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    /// let data = rnx.space_vehicules_per_epoch();
    /// let first_epoch = Epoch::from_gregorian_utc(2017, 1, 1, 0, 0, 0, 0);
    /// let vehicules = data.get(&first_epoch)
    ///     .unwrap();
    /// assert_eq!(*vehicules, vec![
    ///     Sv::new(Constellation::GPS, 03),
    ///     Sv::new(Constellation::GPS, 08),
    ///     Sv::new(Constellation::GPS, 14),
    ///     Sv::new(Constellation::GPS, 16),
    ///     Sv::new(Constellation::GPS, 22),
    ///     Sv::new(Constellation::GPS, 23),
    ///     Sv::new(Constellation::GPS, 26),
    ///     Sv::new(Constellation::GPS, 27),
    ///     Sv::new(Constellation::GPS, 31),
    ///     Sv::new(Constellation::GPS, 32),
    /// ]);
    /// ```
    pub fn space_vehicules_per_epoch(&self) -> BTreeMap<Epoch, Vec<Sv>> {
        let mut map: BTreeMap<Epoch, Vec<Sv>> = BTreeMap::new();
        if let Some(r) = self.record.as_nav() {
            for (epoch, classes) in r {
                let mut inner: Vec<Sv> = Vec::new();
                for (class, frames) in classes.iter() {
                    if *class == navigation::FrameClass::Ephemeris {
                        for frame in frames {
                            let (_, sv, _) = frame.as_eph().unwrap();
                            inner.push(*sv);
                        }
                    }
                }
                if inner.len() > 0 {
                    inner.sort();
                    map.insert(*epoch, inner);
                }
            }
        } else if let Some(r) = self.record.as_obs() {
            for ((epoch, _), (_, vehicules)) in r {
                let mut inner: Vec<Sv> = Vec::new();
                for (sv, _) in vehicules.iter() {
                    inner.push(*sv);
                }
                if inner.len() > 0 {
                    inner.sort();
                    map.insert(*epoch, inner);
                }
            }
        }
        map
    }

    /// Computes and extracts all Satellite elevation and azimuth angles, in degrees,
    /// from all Navigation frames.
    /// Reference position must be determine, either manually passed to this method
    /// (known from external method), or contained in this file header.
    /// Otherwise, it is not possible to calculate such information.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///     .unwrap();
    /// let ref_pos = (3582105.2910_f64, 532589.7313_f64, 5232754.8054_f64);
    /// let sv_angles = rinex.navigation_sat_angles(Some(ref_pos));
    /// for (sv, epochs) in sv_angles {
    ///     for (epoch, (el, azi)) in epochs {
    ///     }
    /// }
    /// ```
    pub fn navigation_sat_angles(
        &self,
        ref_pos: Option<GroundPosition>,
    ) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64)>> {
        let mut ret: HashMap<Sv, BTreeMap<Epoch, (f64, f64)>> = HashMap::new();
        let ref_pos = match ref_pos {
            Some(pos) => pos,
            None => {
                match &self.header.ground_position {
                    Some(pos) => pos.clone(),
                    _ => return ret, // missing ground position
                }
            },
        };
        if let Some(record) = self.record.as_nav() {
            for (epoch, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, sv, ephemeris) = fr.as_eph().unwrap();
                            if let Some((el, az)) = ephemeris.sat_elev_azim(*epoch, ref_pos) {
                                if let Some(data) = ret.get_mut(sv) {
                                    // vehicle already encountered
                                    data.insert(*epoch, (el, az));
                                } else {
                                    let mut map: BTreeMap<Epoch, (f64, f64)> = BTreeMap::new();
                                    map.insert(*epoch, (el, az));
                                    ret.insert(*sv, map);
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /// Computes and extracts from Navigation Data, all satellite positions, in ECEF,
    /// on an epoch basis.
    /// ```
    /// use rinex::prelude::*;
    /// use std::str::FromStr;
    /// use map_3d::{ecef2geodetic, Ellipsoid};
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    /// let g08 = Sv::from_str("G08")
    ///     .unwrap();
    /// rinex.retain_space_vehicule_mut(vec![g08]);
    /// let sat_pos_ecef = rinex.navigation_sat_pos_ecef();
    /// // example: convert to LLA
    /// let sat_pos_lla = rinex.navigation_sat_pos_ecef()
    ///     .iter_mut()
    ///         .map(|(_, epochs)| {
    ///             epochs.iter_mut()
    ///                 .map(|(_, point)| {
    ///                     // convert, overwrite ECEF to LLA coordinates
    ///                     *point = ecef2geodetic(point.0, point.1, point.2, Ellipsoid::WGS84);
    ///                 });
    ///     });
    /// ```
    pub fn navigation_sat_pos_ecef(&self) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> {
        let mut ret: HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> = HashMap::new();
        if let Some(record) = self.record.as_nav() {
            for (epoch, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, sv, ephemeris) = fr.as_eph().unwrap();
                            if let Some(sat_pos) = ephemeris.sat_pos_ecef(*epoch) {
                                if let Some(data) = ret.get_mut(sv) {
                                    // epoch being built
                                    data.insert(*epoch, sat_pos);
                                } else {
                                    // first vehicle for this epoch
                                    let mut map: BTreeMap<Epoch, (f64, f64, f64)> = BTreeMap::new();
                                    map.insert(*epoch, sat_pos);
                                    ret.insert(*sv, map);
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /// Computes and returns all Sv position in the sky, expressed
    /// as (latitude, longitude, altitude), in ddeg and meters
    pub fn navigation_sat_geodetic(&self) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> {
        let mut ret: HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> = HashMap::new();
        if let Some(record) = self.record.as_nav() {
            for (epoch, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, sv, ephemeris) = fr.as_eph().unwrap();
                            if let Some((lat, lon, alt)) = ephemeris.sat_geodetic(*epoch) {
                                if let Some(data) = ret.get_mut(sv) {
                                    data.insert(*epoch, (lat, lon, alt));
                                } else {
                                    let mut map: BTreeMap<Epoch, (f64, f64, f64)> = BTreeMap::new();
                                    map.insert(*epoch, (lat, lon, alt));
                                    ret.insert(*sv, map);
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /*
        /// Computes and extracts from Navigation Data, all satellite
        /// position, speed and accelerations, in ECEF, on an epoch basis
        pub fn navigation_sat_speed_ecef(&self)
            -> BTreeMap<Epoch, HashMap<Sv, ((f64,f64,f64), (f64,f64,f64), (f64,f64,f64))>> {
            let mut ret: BTreeMap<Epoch, HashMap<Sv, ((f64,f64,f64), (f64,f64,f64), (f64,f64,f64))>> = BTreeMap::new();
            let mut pdata: HashMap<Sv, (Epoch, (f64,f64,f64), (f64,f64,f64))> = HashMap::new();
            if let Some(record) = self.record.as_nav() {
                for (epoch, classes) in record {
                    for (class, frames) in classes {
                        if *class == navigation::FrameClass::Ephemeris {
                            for fr in frames {
                                let (_, sv, ephemeris) = fr.as_eph()
                                    .unwrap();
                                if let Some(sat_pos) = ephemeris.sat_pos_ecef(*epoch) {
                                    if let Some(data) = ret.get_mut(epoch) {
                                        // epoch being built
                                        data.insert(*sv, sat_pos);
                                    } else {
                                        // first vehicle for this epoch
                                        let mut map: HashMap<Sv, ((f64,f64,f64), (f64,f64,f64), (f64,f64,f64))> = HashMap::new();
                                        if let Some((p_epoch, p_pos, p_speed)) = pdata.get(sv) {
                                            // sv already encountered
                                            // compute new speed
                                            if let Some((dx, dy, dz)) = ephemeris.sat_speed_ecef(*epoch, *p_pos, *p_epoch) {

                                            }
                                            // compute new accell
                                            if let Some((ddx, ddy, ddz)) = ephemeris.sat_accel_ecef(*epoch, *p_pos, *p_speed, *p_epoch) {

                                            }
                                            map.insert(*sv, (sat_pos, (0.0_f64, 0.0_f64, 0.0_f64), (0.0_f64, 0.0_f64, 0.0_f64));
                                        } else {
                                            // sv never encountered

                                            map.insert(*sv, (sat_pos, (0.0_f64, 0.0_f64, 0.0_f64), (0.0_f64, 0.0_f64, 0.0_f64));
                                        }
                                        ret.insert(*epoch, map);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            ret
        }
    */
    /// Returns list of space vehicules encountered in this file.
    /// For Clocks RINEX: returns list of Vehicules used as reference
    /// in the record.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    /// let vehicules = rnx.space_vehicules();
    /// assert_eq!(vehicules, vec![
    ///     Sv::new(Constellation::GPS, 01),
    ///     Sv::new(Constellation::GPS, 03),
    ///     Sv::new(Constellation::GPS, 06),
    ///     Sv::new(Constellation::GPS, 07),
    ///     Sv::new(Constellation::GPS, 08),
    ///     Sv::new(Constellation::GPS, 09),
    ///     Sv::new(Constellation::GPS, 11),
    ///     Sv::new(Constellation::GPS, 14),
    ///     Sv::new(Constellation::GPS, 16),
    ///     Sv::new(Constellation::GPS, 17),
    ///     Sv::new(Constellation::GPS, 19),
    ///     Sv::new(Constellation::GPS, 22),
    ///     Sv::new(Constellation::GPS, 23),
    ///     Sv::new(Constellation::GPS, 26),
    ///     Sv::new(Constellation::GPS, 27),
    ///     Sv::new(Constellation::GPS, 28),
    ///     Sv::new(Constellation::GPS, 30),
    ///     Sv::new(Constellation::GPS, 31),
    ///     Sv::new(Constellation::GPS, 32),
    /// ]);
    /// ```
    pub fn space_vehicules(&self) -> Vec<Sv> {
        let mut map: Vec<Sv> = Vec::new();
        if let Some(r) = self.record.as_nav() {
            for (_, classes) in r {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for frame in frames {
                            let (_, sv, _) = frame.as_eph().unwrap();
                            if !map.contains(&sv) {
                                map.push(*sv);
                            }
                        }
                    }
                }
            }
        } else if let Some(r) = self.record.as_obs() {
            for (_, (_, vehicules)) in r {
                for (sv, _) in vehicules {
                    if !map.contains(&sv) {
                        map.push(*sv);
                    }
                }
            }
        } else if let Some(r) = self.record.as_clock() {
            for (_, dtypes) in r {
                for (_, systems) in dtypes {
                    for (system, _) in systems {
                        match system {
                            clocks::System::Sv(sv) => {
                                if !map.contains(sv) {
                                    map.push(*sv);
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
        map.sort();
        map
    }

    /// List clocks::System contained in this Clocks RINEX.
    /// Reference systems can either be a Satellite vehicule
    /// or a ground station.
    pub fn clock_ref_systems(&self) -> Vec<clocks::record::System> {
        let mut map: Vec<clocks::record::System> = Vec::new();
        if let Some(r) = self.record.as_clock() {
            for (_, dtypes) in r {
                for (_dtype, systems) in dtypes {
                    for (system, _) in systems {
                        if !map.contains(&system) {
                            map.push(system.clone());
                        }
                    }
                }
            }
        }
        map.sort();
        map
    }

    /// List reference ground stations used in this Clock RINEX.
    /// To list reference Satellite vehicule, simply use [Rinex::space_vehicules].
    pub fn clock_ref_stations(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::with_capacity(32);
        if let Some(r) = self.record.as_clock() {
            for (_, dtypes) in r {
                for (_, systems) in dtypes {
                    for (system, _) in systems {
                        match system {
                            clocks::System::Station(station) => {
                                if !ret.contains(station) {
                                    ret.push(station.clone());
                                }
                            },
                            _ => {},
                        }
                    }
                }
            }
        }
        ret
    }

    /// Lists systems contained in this Clocks RINEX,
    /// on an epoch basis. Systems can either be a ground station or a satellite vehicule.
    pub fn clock_ref_systems_per_epoch(&self) -> BTreeMap<Epoch, Vec<clocks::record::System>> {
        let mut map: BTreeMap<Epoch, Vec<clocks::record::System>> = BTreeMap::new();
        if let Some(r) = self.record.as_clock() {
            for (epoch, dtypes) in r.iter() {
                for (_dtype, systems) in dtypes.iter() {
                    for (system, _) in systems.iter() {
                        if let Some(e) = map.get_mut(epoch) {
                            e.push(system.clone());
                        } else {
                            map.insert(*epoch, vec![system.clone()]);
                        }
                    }
                }
            }
        }
        map
    }

    /// Lists identified Navigation Message types
    pub fn navigation_message_types(&self) -> Vec<navigation::MsgType> {
        let mut ret: Vec<navigation::MsgType> = Vec::new();
        if let Some(record) = self.record.as_nav() {
            for (_, classes) in record.iter() {
                for (class, frames) in classes.iter() {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames.iter() {
                            let (msg, _, _) = fr.as_eph().unwrap();
                            if !ret.contains(msg) {
                                ret.push(*msg);
                            }
                        }
                    } else if *class == navigation::FrameClass::SystemTimeOffset {
                        for fr in frames.iter() {
                            let (msg, _, _) = fr.as_sto().unwrap();
                            if !ret.contains(msg) {
                                ret.push(*msg);
                            }
                        }
                    } else if *class == navigation::FrameClass::IonosphericModel {
                        for fr in frames.iter() {
                            let (msg, _, _) = fr.as_ion().unwrap();
                            if !ret.contains(msg) {
                                ret.push(*msg);
                            }
                        }
                    } else if *class == navigation::FrameClass::EarthOrientation {
                        for fr in frames.iter() {
                            let (msg, _, _) = fr.as_eop().unwrap();
                            if !ret.contains(msg) {
                                ret.push(*msg);
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /// Applies given AND mask in place, to all observations.
    /// This has no effect on non observation records.
    /// This also drops observations that did not come with an LLI flag
    pub fn observation_lli_and_mask_mut(&mut self, mask: observation::LliFlags) {
        if !self.is_observation_rinex() {
            return; // nothing to browse
        }
        let record = self.record.as_mut_obs().unwrap();
        for (_e, (_clk, sv)) in record.iter_mut() {
            for (_sv, obs) in sv.iter_mut() {
                obs.retain(|_, data| {
                    if let Some(lli) = data.lli {
                        lli.intersects(mask)
                    } else {
                        false // drops data with no LLI attached
                    }
                })
            }
        }
    }

    /// Immutable implementation of [observation_lli_and_mask_mut]
    pub fn observation_lli_and_mask(&self, mask: observation::LliFlags) -> Self {
        let mut c = self.clone();
        c.observation_lli_and_mask_mut(mask);
        c
    }

    /// Extracts signal strength as (min, max) duplet,
    /// accross all vehicules.
    /// Only relevant on Observation RINEX.
    pub fn observation_ssi_minmax(&self) -> Option<(observation::Snr, observation::Snr)> {
        let mut ret: Option<(observation::Snr, observation::Snr)> = None;
        if let Some(r) = self.record.as_obs() {
            for (_, (_, vehicules)) in r.iter() {
                for (_, observation) in vehicules.iter() {
                    for (_, data) in observation.iter() {
                        if let Some(snr) = data.snr {
                            if let Some((min, max)) = &mut ret {
                                if snr < *min {
                                    *min = snr;
                                } else if snr > *max {
                                    *max = snr;
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /// Extracts signal strength as (min, max) duplet,
    /// per vehicule. Only relevant on Observation RINEX
    pub fn observation_ssi_sv_minmax(&self) -> HashMap<Sv, (observation::Snr, observation::Snr)> {
        let mut map: HashMap<Sv, (observation::Snr, observation::Snr)> = HashMap::new();
        if let Some(r) = self.record.as_obs() {
            for (_, (_, vehicules)) in r.iter() {
                for (sv, observations) in vehicules.iter() {
                    let (mut min, mut max) = (observation::Snr::DbHz54, observation::Snr::DbHz0);
                    for (_, observation) in observations.iter() {
                        if let Some(ssi) = observation.snr {
                            min = std::cmp::min(min, ssi);
                            max = std::cmp::max(max, ssi);
                        }
                    }
                    map.insert(*sv, (min, max));
                }
            }
        }
        map
    }

    /// Extracts all Ephemeris from this Navigation record,
    /// drops out possible STO / EOP / ION modern NAV frames.  
    /// This does not produce anything if self is not a Navigation RINEX.  
    /// Ephemeris are sorted by epoch and per space vehicule, and comprise
    /// vehicule clock offset, drift, drift rate and other complex data.
    ///
    /// ```
    /// use rinex::*;
    /// // parse a NAV file
    /// let rnx = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx").unwrap();
    /// // extract ephemeris
    /// let ephemeris = rnx.ephemeris();
    /// // browse and exploit ephemeris
    /// for (epoch, vehicules) in ephemeris.iter() {
    ///     for (vehicule, (clk, clk_dr, clk_drr, map)) in vehicules.iter() {
    ///         // clk is the embedded clock bias
    ///         // clk_dr is the embedded clock drift
    ///         // clk_drr is the embedded clock drift rate
    ///         // other data are constellation dependant, refer to db/NAV/navigation.json listing
    ///         let elevation = &map["e"];
    ///         if let Some(elevation) = map.get("e") {
    ///         }   
    ///     }
    /// }
    /// ```
    pub fn ephemeris(
        &self,
    ) -> BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64, HashMap<String, OrbitItem>)>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<
            Epoch,
            BTreeMap<Sv, (f64, f64, f64, HashMap<String, OrbitItem>)>,
        > = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::Ephemeris {
                    let mut inner: BTreeMap<Sv, (f64, f64, f64, HashMap<String, OrbitItem>)> =
                        BTreeMap::new();
                    for frame in frames.iter() {
                        let (_, sv, ephemeris) = frame.as_eph().unwrap();
                        inner.insert(
                            *sv,
                            (
                                ephemeris.clock_bias,
                                ephemeris.clock_drift,
                                ephemeris.clock_drift_rate,
                                ephemeris.orbits.clone(),
                            ),
                        );
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }
    /*
        /// Applies given elevation mask
        pub fn elevation_mask_mut(
            &mut self,
            mask: navigation::ElevationMask,
            ref_pos: Option<(f64, f64, f64)>,
        ) {
            let ref_pos = match ref_pos {
                Some(ref_pos) => ref_pos,
                _ => self.header.coords.expect(
                    "can't apply an elevation mask when ground/ref position is unknown.
    Specify one yourself with `ref_pos`",
                ),
            };
            if let Some(r) = self.record.as_mut_nav() {
                r.retain(|epoch, classes| {
                    classes.retain(|class, frames| {
                        if *class == navigation::FrameClass::Ephemeris {
                            frames.retain(|fr| {
                                let (_, _, ephemeris) = fr.as_eph().unwrap();
                                if let Some((el, _)) = ephemeris.sat_elev_azim(*epoch, ref_pos) {
                                    mask.fits(el)
                                } else {
                                    false
                                }
                            });
                            frames.len() > 0
                        } else {
                            // not an EPH
                            true // keep it anyway
                        }
                    });
                    classes.len() > 0
                })
            }
        }
    */
    /*
        pub fn elevation_mask(
            &self,
            mask: navigation::ElevationMask,
            ref_pos: Option<(f64, f64, f64)>,
        ) -> Self {
            let mut s = self.clone();
            s.elevation_mask_mut(mask, ref_pos);
            s
        }
    */
    /// Extracts all System Time Offset data
    /// on a epoch basis, from this Navigation record.
    /// This does not produce anything if self is not a modern Navigation record
    /// that contains such frames.
    pub fn navigation_system_time_offsets(&self) -> BTreeMap<Epoch, Vec<navigation::StoMessage>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::StoMessage>> = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::SystemTimeOffset {
                    let mut inner: Vec<navigation::StoMessage> = Vec::new();
                    for frame in frames.iter() {
                        let (_, _, fr) = frame.as_sto().unwrap();
                        inner.push(fr.clone())
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }

    /// Extracts from this Navigation record all Ionospheric Models, on a epoch basis,
    /// regardless of their kind. This does not produce anything if
    /// self is not a modern Navigation record that contains such models.
    pub fn navigation_ionospheric_models(&self) -> BTreeMap<Epoch, Vec<navigation::IonMessage>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::IonMessage>> = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner: Vec<navigation::IonMessage> = Vec::new();
                    for frame in frames.iter() {
                        let (_, _, fr) = frame.as_ion().unwrap();
                        inner.push(fr.clone())
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }

    /// Extracts all Klobuchar Ionospheric models from this Navigation record.
    /// This does not produce anything if self is not a modern Navigation record
    /// that contains such models.
    pub fn navigation_klobuchar_ionospheric_models(
        &self,
    ) -> BTreeMap<Epoch, Vec<navigation::KbModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::KbModel>> = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner: Vec<navigation::KbModel> = Vec::new();
                    for frame in frames.iter() {
                        let (_, _, fr) = frame.as_ion().unwrap();
                        if let Some(model) = fr.as_klobuchar() {
                            inner.push(*model);
                        }
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }

    /// Extracts all Nequick-G Ionospheric models from this Navigation record.
    /// This does not produce anything if self is not a modern Navigation record
    /// that contains such models.
    pub fn navigation_nequickg_ionospheric_models(
        &self,
    ) -> BTreeMap<Epoch, Vec<navigation::NgModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::NgModel>> = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner: Vec<navigation::NgModel> = Vec::new();
                    for frame in frames.iter() {
                        let (_, _, fr) = frame.as_ion().unwrap();
                        if let Some(model) = fr.as_nequick_g() {
                            inner.push(*model);
                        }
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }

    /// Extracts all BDGIM Ionospheric models from this Navigation record.
    /// This does not produce anything if self is not a modern Navigation record
    /// that contains such models.
    pub fn navigation_bdgim_ionospheric_models(&self) -> BTreeMap<Epoch, Vec<navigation::BdModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::BdModel>> = BTreeMap::new();
        let record = self.record.as_nav().unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner: Vec<navigation::BdModel> = Vec::new();
                    for frame in frames.iter() {
                        let (_, _, fr) = frame.as_ion().unwrap();
                        if let Some(model) = fr.as_bdgim() {
                            inner.push(*model);
                        }
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }
    /// Aligns Phase observations at origins
    pub fn observation_align_phase_origins_mut(&mut self) {
        let mut init_phases: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicules)) in r.iter_mut() {
                for (sv, observations) in vehicules.iter_mut() {
                    for (observable, data) in observations.iter_mut() {
                        if observable.is_phase_observable() {
                            if let Some(init_phase) = init_phases.get_mut(&sv) {
                                if init_phase.get(observable).is_none() {
                                    init_phase.insert(observable.clone(), data.obs);
                                }
                            } else {
                                let mut map: HashMap<Observable, f64> = HashMap::new();
                                map.insert(observable.clone(), data.obs);
                                init_phases.insert(*sv, map);
                            }
                            data.obs -= init_phases.get(&sv).unwrap().get(observable).unwrap();
                        }
                    }
                }
            }
        }
    }
    /// Aligns Phase observations at origins,
    /// immutable implementation
    pub fn observation_align_phase_origins(&self) -> Self {
        let mut s = self.clone();
        s.observation_align_phase_origins_mut();
        s
    }
	/// Form desired signal combinations
	pub fn observation_combination(&self, combination: Combination) ->  HashMap<(Observable, Observable), HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
		if let Some(r) = self.record.as_obs() {
			r.combine(combination)
		} else {
			HashMap::new()
		}
	}
	/// GNSS (differential) code biases
	pub fn observation_dcb(&self) -> HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
		if let Some(r) = self.record.as_obs() {
			r.dcb()
		} else {
			HashMap::new()
		}
	}
    /// Ionospheric delay detector
	pub fn observation_iono_delay_detector(&self) ->  HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> {
		if let Some(r) = self.record.as_obs() {
			if let Some(dt) = self.sampling_interval() {
				r.iono_delay_detector(dt)
			} else {
				HashMap::new()
			}
		} else {
			HashMap::new()
		}
	}
	/// Code multipath analysis (MP_i), cf.
    /// phase data model <https://github.com/gwbres/rinex/blob/main/rinex-cli/doc/gnss-combination.md>.
    pub fn observation_code_multipath(
        &self,
    ) -> HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        let mut ret: HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> =
            HashMap::new();
        if let Some(record) = self.record.as_obs() {
            /*
             * Determine mean value of all datasets
             */
            let mut mean: HashMap<Sv, HashMap<Observable, (u32, f64)>> = HashMap::new();
            for (_epoch, (_, vehicles)) in record {
                for (sv, observations) in vehicles {
                    if let Some(data) = mean.get_mut(&sv) {
                        for (observable, obs_data) in observations {
                            if observable.is_phase_observable()
                                || observable.is_pseudorange_observable()
                            {
                                if let Some((count, buf)) = data.get_mut(observable) {
                                    *count += 1;
                                    *buf += obs_data.obs;
                                } else {
                                    data.insert(observable.clone(), (1, obs_data.obs));
                                }
                            }
                        }
                    } else {
                        for (observable, obs_data) in observations {
                            if observable.is_phase_observable()
                                || observable.is_pseudorange_observable()
                            {
                                let mut map: HashMap<Observable, (u32, f64)> = HashMap::new();
                                map.insert(observable.clone(), (1, obs_data.obs));
                                mean.insert(*sv, map);
                            }
                        }
                    }
                }
            }
            mean.iter_mut()
                .map(|(_, data)| {
                    data.iter_mut()
                        .map(|(_, (n, buf))| {
                            *buf = *buf / *n as f64;
                        })
                        .count()
                })
                .count();
            //println!("MEAN VALUES {:?}", mean); //DEBUG
            /*
             * Run algorithm
             */
            let mut associated: HashMap<String, String> = HashMap::new(); // Ph code to associate to this Mpx
                                                                          // for operation consistency
            for (epoch, (_, vehicules)) in record {
                for (sv, observations) in vehicules {
                    let mean_sv = mean.get(&sv).unwrap();
                    for (lhs_observable, lhs_data) in observations {
                        if lhs_observable.is_pseudorange_observable() {
                            let pr_i = lhs_data.obs; // - mean_sv.get(lhs_code).unwrap().1;
                            let lhs_code = lhs_observable.to_string();
                            let mp_code = &lhs_code[2..]; //TODO will not work on RINEX2
                            let lhs_carrier = &lhs_code[1..2];
                            let mut ph_i: Option<f64> = None;
                            let mut ph_j: Option<f64> = None;
                            /*
                             * This will restrict combinations to
                             * 1 => 2
                             * and M => 1
                             */
                            let rhs_carrier = match lhs_carrier {
                                "1" => "2",
                                _ => "1",
                            };
                            /*
                             * locate related L_i PH code
                             */
                            for (observable, data) in observations {
                                let ph_code = format!("L{}", mp_code);
                                let code = observable.to_string();
                                if code.eq(&ph_code) {
                                    ph_i = Some(data.obs); // - mean_sv.get(code).unwrap().1);
                                    break; // DONE
                                }
                            }
                            /*
                             * locate another L_j PH code
                             */
                            if let Some(to_locate) = associated.get(mp_code) {
                                /*
                                 * We already have an association, keep it consistent throughout
                                 * operations
                                 */
                                for (observable, data) in observations {
                                    let code = observable.to_string();
                                    let carrier_code = &code[1..2];
                                    if carrier_code == rhs_carrier {
                                        // correct carrier signal
                                        if code.eq(to_locate) {
                                            // match
                                            ph_j = Some(data.obs); // - mean_sv.get(code).unwrap().1);
                                            break; // DONE
                                        }
                                    }
                                }
                            } else {
                                // first: prefer the same code against rhs carrier
                                let to_locate = format!("L{}{}", rhs_carrier, &mp_code[1..]);
                                for (observable, data) in observations {
                                    let code = observable.to_string();
                                    let carrier_code = &code[1..2];
                                    if carrier_code == rhs_carrier {
                                        // correct carrier
                                        if code.eq(&to_locate) {
                                            // match
                                            ph_j = Some(data.obs); // - mean_sv.get(code).unwrap().1);
                                            associated.insert(mp_code.to_string(), code.clone());
                                            break; // DONE
                                        }
                                    }
                                }
                                if ph_j.is_none() {
                                    /*
                                     * Same code against different carrier does not exist
                                     * try to grab another PH code, against rhs carrier
                                     */
                                    for (observable, data) in observations {
                                        let code = observable.to_string();
                                        let carrier_code = &code[1..2];
                                        if carrier_code == rhs_carrier {
                                            if observable.is_phase_observable() {
                                                ph_j = Some(data.obs); // - mean_sv.get(code).unwrap().1);
                                                associated
                                                    .insert(mp_code.to_string(), code.clone());
                                                break; // DONE
                                            }
                                        }
                                    }
                                }
                            }
                            if ph_i.is_none() || ph_j.is_none() {
                                break; // incomplete associations, do not proceed further
                            }
                            let ph_i = ph_i.unwrap();
                            let ph_j = ph_j.unwrap();
                            let lhs_carrier = lhs_observable.carrier(sv.constellation).unwrap();
                            let rhs_carrier =
                                lhs_observable //rhs_observable TODO
                                    .carrier(sv.constellation)
                                    .unwrap();
                            /*let gamma = (lhs_carrier.carrier_frequency() / rhs_carrier.carrier_frequency()).powf(2.0);
                            let alpha = (gamma +1.0_f64) / (gamma - 1.0_f64);
                            let beta = 2.0_f64 / (gamma - 1.0_f64);
                            let mp = pr_i - alpha * ph_i + beta * ph_j;*/

                            let alpha = 2.0_f64 * rhs_carrier.carrier_frequency().powf(2.0)
                                / (lhs_carrier.carrier_frequency().powf(2.0)
                                    - rhs_carrier.carrier_frequency().powf(2.0));
                            let mp = pr_i - ph_i - alpha * (ph_i - ph_j);
                            if let Some(data) = ret.get_mut(mp_code) {
                                if let Some(data) = data.get_mut(&sv) {
                                    data.insert(*epoch, mp);
                                } else {
                                    let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> =
                                        BTreeMap::new();
                                    let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                        HashMap::new();
                                    bmap.insert(*epoch, mp);
                                    data.insert(*sv, bmap);
                                }
                            } else {
                                let mut bmap: BTreeMap<(Epoch, EpochFlag), f64> = BTreeMap::new();
                                let mut map: HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>> =
                                    HashMap::new();
                                bmap.insert(*epoch, mp);
                                map.insert(*sv, bmap);
                                ret.insert(mp_code.to_string(), map);
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /*
        /// Single step /stage, in high order phase differencing
        /// algorithm, which we use in case of old receiver data / old RINEX
        /// to cancel geometric and atmospheric biases.
        /// See [high_order_phase_difference]
        fn high_order_phase_difference_step(&self) -> Result<BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>> {
            let mut ret: BTreeMap<Epoch, HashMap<String, f64>> = BTreeMap::new();
        }

        /// Computes High Order Phase Difference
        /// accross vehicules and epochs,
        /// until differencing order is reached.
        /// This is used in Geometric biases estimation,
        /// in case of single channel receivers / old RINEX, where
        /// only one carrier signal was sampled.
        /// Final order is determined from the epoch interval
        /// (the smallest the better), the phase data quality and so on.
        fn high_order_phase_difference(&self, order: usize) -> Result<BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>> {
            let mut ret: BTreeMap<Epoch, HashMap<String, f64>> = BTreeMap::new();
            if let Some(rec) = self.record.as_obs() {
                for (epoch, (_, vehicules)) in rec {
                    for (sv, observations) in vehicules {
                        for (code, data) in observations {
                            if is_phase_observation(code) {

                            }
                        }
                    }
                }
            }
            ret
        }

        /// Estimates geometric biases () in Eq(2) page 2
        /// of <>, which is the dominant bias in the cycle slip
        /// determination. This is performed by substracting
        /// raw phase data measurement for a given observation,
        /// against the same observation along separate carrier frequency.
        /// It is said to
    */
    /*
        /// Extracts Pseudo Ranges without Ionospheric path delay contributions,
        /// by extracting [pseudo_ranges] and using the differential (dual frequency) compensation.
        /// We can only compute such information if pseudo range was evaluted
        /// on at least two seperate carrier frequencies, for a given space vehicule at a certain epoch.
        /// Does not produce anything if self is not an Observation RINEX.
        pub fn iono_free_pseudo_ranges (&self) -> BTreeMap<Epoch, BTreeMap<Sv, f64>> {
            let pr = self.observation_pseudo_ranges();
            let mut results : BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
            for (e, sv) in pr.iter() {
                let mut map :BTreeMap<Sv, f64> = BTreeMap::new();
                for (sv, obs) in sv.iter() {
                    let mut result :Option<f64> = None;
                    let mut retained : Vec<(String, f64)> = Vec::new();
                    for (code, value) in obs.iter() {
                        if is_pseudorange_observation(code) {
                            retained.push((code.clone(), *value));
                        }
                    }
                    if retained.len() > 1 { // got a dual frequency scenario
                        // we only care about 2 carriers
                        let retained = &retained[0..2];
                        // only left with two observables at this point
                        // (obscode, data) mapping
                        let codes :Vec<String> = retained.iter().map(|r| r.0.clone()).collect();
                        let data :Vec<f64> = retained.iter().map(|r| r.1).collect();
                        // need to determine frequencies involved
                        let mut channels :Vec<Carrier> = Vec::with_capacity(2);
                        for i in 0..codes.len() {
                            if let Ok(channel) = Carrier::from_observable(sv.constellation, &codes[i]) {
                                channels.push(channel)
                            }
                        }
                        if channels.len() == 2 { // frequency identification passed, twice
                            // --> compute
                            let f0 = (channels[0].carrier_frequency_mhz() *1.0E6).powf(2.0_f64);
                            let f1 = (channels[1].carrier_frequency_mhz() *1.0E6).powf(2.0_f64);
                            let diff = (f0 * data[0] - f1 * data[1] ) / (f0 - f1) ;
                            result = Some(diff)
                        }
                    }
                    if let Some(result) = result {
                        // conditions were met for this vehicule
                        // at this epoch
                        map.insert(*sv, result);
                    }
                }
                if map.len() > 0 { // did produce something
                    results.insert(*e, map);
                }
            }
            results
        }
    */
    /*
        /// Extracts Raw Carrier Phase observations,
        /// from this Observation record, on an epoch basis an per space vehicule.
        /// Does not produce anything if self is not an Observation RINEX.
        pub fn observation_phase(&self) -> BTreeMap<(Epoch, EpochFlag), HashMap<Sv, Vec<(String, f64)>>> {
            let mut ret: BTreeMap<Epoch, BTreeMap<Sv, Vec<(String, f64)>>> = BTreeMap::new();
            if let Some(r) = self.record.as_obs() {
                for ((e, _), (_, sv)) in record.iter() {
                    let mut map: BTreeMap<Sv, Vec<(String, f64)>> = BTreeMap::new();
                    for (sv, obs) in sv.iter() {
                        let mut v: Vec<(String, f64)> = Vec::new();
                        for (observable, data) in obs.iter() {
                            if observable.is_phase_observation() {
                                v.push((code.clone(), data.obs));
                            }
                        }
                        if v.len() > 0 {
                            // did come with at least 1 Phase obs
                            map.insert(*sv, v);
                        }
                    }
                    if map.len() > 0 {
                        // did produce something
                        results.insert(*e, map);
                    }
                }
            }
            ret
        }
    */
    /*
        /// Extracts Carrier phases without Ionospheric path delay contributions,
        /// by extracting [carrier_phases] and using the differential (dual frequency) compensation.
        /// We can only compute such information if carrier phase was evaluted
        /// on at least two seperate carrier frequencies, for a given space vehicule at a certain epoch.
        /// Does not produce anything if self is not an Observation RINEX.
        pub fn iono_free_observation_carrier_phases (&self) -> BTreeMap<Epoch, BTreeMap<Sv, f64>> {
            let pr = self.observation_pseudoranges();
            let mut results : BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
            for (e, sv) in pr.iter() {
                let mut map :BTreeMap<Sv, f64> = BTreeMap::new();
                for (sv, obs) in sv.iter() {
                    let mut result :Option<f64> = None;
                    let mut retained : Vec<(String, f64)> = Vec::new();
                    for (code, value) in obs.iter() {
                        if is_phase_observation(code) {
                            retained.push((code.clone(), *value));
                        }
                    }
                    if retained.len() > 1 { // got a dual frequency scenario
                        // we only care about 2 carriers
                        let retained = &retained[0..2];
                        // only left with two observables at this point
                        // (obscode, data) mapping
                        let codes :Vec<String> = retained.iter().map(|r| r.0.clone()).collect();
                        let data :Vec<f64> = retained.iter().map(|r| r.1).collect();
                        // need to determine frequencies involved
                        let mut channels :Vec<Carrier> = Vec::with_capacity(2);
                        for i in 0..codes.len() {
                            if let Ok(channel) = Carrier::from_observable(sv.constellation, &codes[i]) {
                                channels.push(channel)
                            }
                        }
                        if channels.len() == 2 { // frequency identification passed, twice
                            // --> compute
                            let f0 = (channels[0].carrier_frequency_mhz() *1.0E6).powf(2.0_f64);
                            let f1 = (channels[1].carrier_frequency_mhz() *1.0E6).powf(2.0_f64);
                            let diff = (f0 * data[0] - f1 * data[1] ) / (f0 - f1) ;
                            result = Some(diff)
                        }
                    }
                    if let Some(result) = result {
                        // conditions were met for this vehicule
                        // at this epoch
                        map.insert(*sv, result);
                    }
                }
                if map.len() > 0 { // did produce something
                    results.insert(*e, map);
                }
            }
            results
        }
    */
    /*
        /// Returns all Pseudo Range observations
        /// converted to Real Distance (in [m]),
        /// by compensating for the difference between
        /// local clock offset and distant clock offsets.
        /// We can only produce such data if local clock offset was found
        /// for a given epoch, and related distant clock offsets were given.
        /// Distant clock offsets can be obtained with [space_vehicules_clock_offset].
        /// Real distances are extracted on an epoch basis, and per space vehicule.
        /// This method has no effect on non observation data.
        ///
        /// Example:
        /// ```
        /// use rinex::prelude::*;
        /// // obtain distance clock offsets, by analyzing a related NAV file
        /// // (this is only an example..)
        /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
        ///     .unwrap();
        /// // Retain G07 + G08 vehicules
        /// // to perform further calculations on these vehicules data (GPS + Svnn filter)
        /// let filter = vec![
        ///     Sv {
        ///         constellation: Constellation::GPS,
        ///         prn: 7,
        ///     },
        ///     Sv {
        ///         constellation: Constellation::GPS,
        ///         prn: 8,
        ///     },
        /// ];
        /// rinex
        ///     .retain_space_vehicule_mut(filter.clone());
        /// // extract distant clock offsets
        /// let sv_clk_offsets = rinex.space_vehicules_clock_offset();
        /// let rinex = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx");
        /// let mut rinex = rinex.unwrap();
        /// // apply the same filter
        /// rinex
        ///     .retain_space_vehicule_mut(filter.clone());
        /// let distances = rinex.observation_pseudodistances(sv_clk_offsets);
        /// // exploit distances
        /// for (e, sv) in distances.iter() { // (epoch, vehicules)
        ///     for (sv, obs) in sv.iter() { // (vehicule, distance)
        ///         for ((code, distance)) in obs.iter() { // obscode, distance
        ///             // use the 3 letter code here,
        ///             // to determine the carrier you're dealing with.
        ///             let d = distance * 10.0; // consume, post process...
        ///         }
        ///     }
        /// }
        /// ```
        pub fn observation_pseudodistances(
            &self,
            sv_clk_offsets: BTreeMap<Epoch, BTreeMap<Sv, f64>>,
        ) -> BTreeMap<(Epoch, EpochFlag), BTreeMap<Sv, Vec<(String, f64)>>> {
            let mut results: BTreeMap<(Epoch, EpochFlag), BTreeMap<Sv, Vec<(String, f64)>>> =
                BTreeMap::new();
            if let Some(r) = self.record.as_obs() {
                for ((e, flag), (clk, sv)) in r {
                    if let Some(distant_e) = sv_clk_offsets.get(e) {
                        // got related distant epoch
                        if let Some(clk) = clk {
                            // got local clock offset
                            let mut map: BTreeMap<Sv, Vec<(String, f64)>> = BTreeMap::new();
                            for (sv, obs) in sv.iter() {
                                if let Some(sv_offset) = distant_e.get(sv) {
                                    // got related distant offset
                                    let mut v: Vec<(String, f64)> = Vec::new();
                                    for (observable, data) in obs.iter() {
                                        if observable.is_pseudorange_observation() {
                                            // We currently do not support the compensation for biases
                                            // than clock induced ones. ie., Ionospheric delays ??
                                            v.push((
                                                observable.code().unwrap(),
                                                data.pr_real_distance(*clk, *sv_offset, 0.0),
                                            ));
                                        }
                                    }
                                    if v.len() > 0 {
                                        // did come with at least 1 PR
                                        map.insert(*sv, v);
                                    }
                                } // got related distant offset
                            } // per sv
                            if map.len() > 0 {
                                // did produce something
                                results.insert((*e, *flag), map);
                            }
                        } // got local clock offset attached to this epoch
                    } //got related distance epoch
                } // per epoch
            }
            results
        }
    */
    /*
        /// Applies Hatch filter to all Pseudo Range observations.
        /// When feasible dual frequency dual code method is prefered
        /// for optimal, fully unbiased smoothed PR.
        /// PR observations get modified in place
        pub fn observation_pseudorange_smoothing_mut(&mut self) {
            if let Some(r) = self.record.as_mut_obs() {
                for ((epoch, _), (_, svs)) in r {
                    for (sv, observations) in svs {
                        for (code, observation) in observations {

                        }
                    }
                }
            }
        }

        pub fn observation_pseudorange_smoothing(&self) -> Self {
            let mut s = self.clone();
            s.observation_pseudorange_smoothing();
            s
        }
    */

    /// Restrain epochs to interval |start <= e <= end| (both included)
    pub fn time_window_mut(&mut self, start: Epoch, end: Epoch) {
        if let Some(record) = self.record.as_mut_obs() {
            record.retain(|(e, _), _| e >= &start && e <= &end);
        } else if let Some(record) = self.record.as_mut_nav() {
            record.retain(|e, _| e >= &start && e <= &end);
        } else if let Some(record) = self.record.as_mut_meteo() {
            record.retain(|e, _| e >= &start && e <= &end);
        } else if let Some(record) = self.record.as_mut_clock() {
            record.retain(|e, _| e >= &start && e <= &end);
        } else if let Some(record) = self.record.as_mut_ionex() {
            record.retain(|e, _| e >= &start && e <= &end);
        }
    }

    /// Returns a copy version of `self` where epochs were constrained
    /// to |start <= e <= end| interval (both included)
    pub fn time_window(&self, start: Epoch, end: Epoch) -> Self {
        let mut s = self.clone();
        s.time_window_mut(start, end);
        s
    }

    /*
        /// Returns epochs where a so called "cycle slip" has been confirmed.
        /// We confirm a cycle slip by computing the double difference
        /// between self and `rhs` Observation RINEX.
        /// This does not produce anything if both are not
        /// Observation RINEX files, ideally sampled at same epochs,
        /// by two seperate but stationnary receivers. Refer to [diff_mut]
        /// for further explanations on those computations.
        ///
        /// Example:
        /// ```
        /// use rinex::*;
        /// // grab an observation rinex
        /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o").unwrap();
        /// let cycle_slips = rnx.cycle_slips();
        ///
        /// // now we will confirm those cycle slip events by computing the double diff,
        /// // assuming this secondary rinex recorded the same data
        /// let rnx_b = Rinex::from_file("../test_resources/OBS/V2/npaz3550.21o").unwrap();
        /// let confirmed_slips = rnx.confirmed_cycle_slips(&rnx_b);
        /// ```
        pub fn observation_confirmed_cycle_slip_epochs (&self, rhs: &Self) -> Result<Vec<Epoch>, DiffError> {
            if !self.is_observation_rinex() || !rhs.is_observation_rinex() {
                return Err(DiffError::NotObsRinex) ;
            }
            let rnx = self.double_diff(rhs);
            let vec : Vec<Epoch> = Vec::new();
            Ok(vec)
        }
    CYCLE SLIPS CONFIRMATION
    */

    /// Filters out data that was not produced by given agency / station.
    /// This has no effect on records other than CLK RINEX.
    /// Example:
    /// ```
    /// use rinex::*;
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/CLK/V2/COD20352.CLK")
    ///         .unwrap();
    /// rinex
    ///     .clock_agency_retain_mut(vec!["GUAM","GODE","USN7"]);
    /// ```
    pub fn clock_agency_retain_mut(&mut self, filter: Vec<&str>) {
        if !self.is_clocks_rinex() {
            return; // does not apply
        }
        let record = self.record.as_mut_clock().unwrap();
        record.retain(|_, dtypes| {
            dtypes.retain(|_dtype, systems| {
                systems.retain(|system, _| {
                    if let Some(agency) = system.as_station() {
                        filter.contains(&agency.as_str())
                    } else {
                        false
                    }
                });
                systems.len() > 0
            });
            dtypes.len() > 0
        })
    }
    /// Writes self into given file.   
    /// Both header + record will strictly follow RINEX standards.   
    /// Record: refer to supported RINEX types
    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut writer = BufferedWriter::new(path)?;
        write!(writer, "{}", self.header.to_string())?;
        self.record.to_file(&self.header, &mut writer)?;
        Ok(())
    }
}

impl Merge<Rinex> for Rinex {
    /// Merges `rhs` into `Self` without mutable access, at the expense of memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self` in place
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        self.header.merge_mut(&rhs.header)?;
        if self.epochs().len() == 0 {
            // self is empty
            self.record = rhs.record.clone();
            Ok(())
        } else if rhs.epochs().len() == 0 {
            // nothing to merge
            Ok(())
        } else {
            // add special marker, ts: YYYYDDMM HHMMSS UTC
            let now = hifitime::Epoch::now().expect("failed to retrieve system time");
            let (y, m, d, hh, mm, ss, _) = now.to_gregorian_utc();
            self.header.comments.push(format!(
                "rustrnx-{:<20} FILE MERGE          {}{}{} {}{}{} {}",
                env!("CARGO_PKG_VERSION"),
                y + 1900,
                m,
                d,
                hh,
                mm,
                ss,
                now.time_scale
            ));
            // RINEX record merging
            self.record.merge_mut(&rhs.record)?;
            Ok(())
        }
    }
}

impl Split<Rinex> for Rinex {
    /// Splits `Self` at desired epoch
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let (r0, r1) = self.record.split(epoch)?;
        Ok((
            Self {
                header: self.header.clone(),
                comments: self.comments.clone(),
                record: r0,
            },
            Self {
                header: self.header.clone(),
                comments: self.comments.clone(),
                record: r1,
            },
        ))
    }
	fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, split::Error> {
		Ok(Vec::new())
	}
}

impl Decimate<Rinex> for Rinex {
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        self.record.decimate_by_ratio_mut(r);
        if let Some(_) = self.header.sampling_interval {
            self.header.sampling_interval = Some(
                //update
                self.header.sampling_interval.unwrap() / r as f64,
            );
        }
    }
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    fn decimate_by_interval_mut(&mut self, interval: Duration) {
        self.record.decimate_by_interval_mut(interval);
        if let Some(_) = self.header.sampling_interval {
            self.header.sampling_interval = Some(interval);
        }
    }
    fn decimate_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(interval);
        s
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        self.record.decimate_match_mut(&rhs.record);
        if self.header.sampling_interval.is_some() {
            if let Some(b) = rhs.header.sampling_interval {
                self.header.sampling_interval = Some(b);
            }
        }
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(&rhs);
        s
    }
}

impl Smooth<Rinex> for Rinex {
	fn hatch_smoothing(&self) -> Self {
		let mut s = self.clone();
		s.hatch_smoothing_mut();
		s
	}
	fn hatch_smoothing_mut(&mut self) {
		if let Some(r) = self.record.as_mut_obs() {
			r.hatch_smoothing_mut();
		}
	}
}

impl TimeScaling<Rinex> for Rinex {
    fn convert_timescale(&mut self, ts: TimeScale) {
        self.record.convert_timescale(ts);
    }
    fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.convert_timescale(ts);
        s
    }
}

use crate::algorithm::{Preprocessing, Filter};

impl Preprocessing for Rinex {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
        self.record.filter_mut(f);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_macros() {
        assert_eq!(is_comment!("This is a comment COMMENT"), true);
        assert_eq!(is_comment!("This is a comment"), false);
    }
    #[test]
    fn test_hourly_session() {
        assert_eq!(hourly_session!(0), "a");
        assert_eq!(hourly_session!(1), "b");
        assert_eq!(hourly_session!(2), "c");
        assert_eq!(hourly_session!(3), "d");
        assert_eq!(hourly_session!(4), "e");
        assert_eq!(hourly_session!(5), "f");
        assert_eq!(hourly_session!(23), "x");
    }
}
