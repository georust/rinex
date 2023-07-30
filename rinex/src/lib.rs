//! This library provides a set of tools to parse, analyze
//! and process RINEX files.
//! Refer to README and documentation provided here
//! <https://github.com/georust/rinex>
#![cfg_attr(docrs, feature(doc_cfg))]

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

mod ground_position;
mod leap;
mod observable;

#[macro_use]
mod macros;

#[cfg(feature = "tests")]
#[cfg_attr(docrs, doc(cfg(feature = "tests")))]
pub mod test_toolkit;

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
use thiserror::Error;

use hifitime::Duration;
use observable::Observable;
use observation::Crinex;
use version::Version;

/// Package to include all basic structures
pub mod prelude {
    pub use crate::constellation::{Augmentation, Constellation};
    pub use crate::epoch::EpochFlag;
    pub use crate::ground_position::GroundPosition;
    pub use crate::header::Header;
    pub use crate::observable::Observable;
    pub use crate::sv::Sv;
    pub use crate::Rinex;
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries};
}

/// Processing package,
/// includes preprocessing methods and analysis methods.
pub mod processing {
    pub use crate::algorithm::*;
    //pub use differential::*;
    //pub use crate::cs::{CsDetector, CsSelectionMethod, CsStrategy};
}

#[cfg(feature = "qc")]
mod qc;

#[cfg(feature = "qc")]
#[macro_use]
extern crate horrorshow;

/// Quality analysis package: mainly statistical analysis on OBS RINEX
#[cfg(feature = "qc")]
#[cfg_attr(docrs, doc(cfg(feature = "qc")))]
pub mod quality {
    pub use crate::qc::{HtmlReport, QcOpts, QcReport};
}

use carrier::Carrier;
use gnss_time::GnssTime;
use prelude::*;

pub use merge::Merge;
pub use split::Split;

mod algorithm;
use algorithm::{IonoDelayDetector, Smooth};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

/// File creation helper.
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

#[derive(Clone, Default, Debug, PartialEq)]
/// `Rinex` describes a `RINEX` file, it comprises a [Header] section,
/// and a [record::Record] file body.   
/// This parser can also store comments encountered while parsing the file body,
/// stored as [record::Comments], without much application other than presenting
/// all encountered data at the moment.   
/// Following is an example of high level usage (mainly header fields).  
/// For RINEX type dependent usage, refer to related [record::Record] definition.  
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
/// // Constellation describes which kind of vehicles
/// // are to be encountered in the record, or which
/// // GNSS constellation the data will be referred to.
/// // Mixed constellation, means a combination of vehicles or
/// // GNSS constellations is expected
/// assert_eq!(rnx.header.constellation, Some(Constellation::Mixed));
/// // Some information on the hardware being used might be stored
/// println!("{:#?}", rnx.header.rcvr);
/// // WGS84 receiver approximate position
/// println!("{:#?}", rnx.header.ground_position);
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
    ///
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    ///
    /// // convert to CRINEX
    /// let crinex = rinex.rnx2crnx();
    /// // generate
    /// crinex.to_file("test.crx")
    ///     .unwrap();
    /// ```
    pub fn rnx2crnx(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx_mut();
        s
    }

    /// [Rinex::rnx2crnx] mutable implementation
    pub fn rnx2crnx_mut(&mut self) {
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
    pub fn rnx2crnx1(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx1_mut();
        s
    }

    /// [Rinex::rnx2crnx1] mutable implementation.
    pub fn rnx2crnx1_mut(&mut self) {
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
    pub fn rnx2crnx3(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx1_mut();
        s
    }

    /// [Rinex::rnx2crnx3] mutable implementation.
    pub fn rnx2crnx3_mut(&mut self) {
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

    /// Forms timeseries from all Epochs contained in this record
    pub fn timeseries(&self) -> Option<TimeSeries> {
        if let Some(dt) = self.sampling_interval() {
            Some(self.record.timeseries(dt))
        } else {
            None
        }
    }

    /// Converts self into given timescale
    pub fn into_timescale(&mut self, ts: TimeScale) {
        self.record.convert_timescale(ts);
    }

    /// Converts self to given timescale
    pub fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.into_timescale(ts);
        s
    }

    /// Converts a CRINEX (compressed RINEX) into readable RINEX.
    /// This has no effect if self is not an Observation RINEX.
    pub fn crnx2rnx(&self) -> Self {
        let mut s = self.clone();
        s.crnx2rnx_mut();
        s
    }

    /// [Rinex::crnx2rnx] mutable implementation
    pub fn crnx2rnx_mut(&mut self) {
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

    /// File creation helper, returns a filename that would respect
    /// naming conventions, based on self attributes.
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

    /// Returns true if Self is a IONEX
    pub fn is_ionex(&self) -> bool {
        self.header.rinex_type == types::Type::IonosphereMaps
    }

    /// Returns true if Self is a 3D IONEX.  
    /// In this case, you can have TEC values at different altitudes, for a given Epoch.
    pub fn is_ionex_3d(&self) -> bool {
        if let Some(ionex) = &self.header.ionex {
            ionex.map_dimension == 3
        } else {
            false
        }
    }

    /// Returns true if Self is a 2D IONEX.
    /// In this case, all TEC values are presented at the same altitude points.
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
    /// This is most often highly dependent on receiver hardware
    /// and receiving conditions.
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

    /// Returns Epochs where unexpected data gap took place.   
    /// Data gap is determined by comparing |e(k) - e(k-1)|,   
    /// the instantaneous epoch interval, to the dominant sampling interval
    /// determined by [Rinex::sampling_interval].
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

    /// Returns list of epoch where an anomaly is reported by the receiver.   
    /// Refer to [epoch::EpochFlag] definitions to see all possible
    /// receiver anomalies. This is only relevant on OBS RINEX.
    pub fn epoch_anomalies(&self) -> Vec<Epoch> {
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

    /// Returns Epochs where a loss of lock event happened.   
    /// This is only relevant on OBS RINEX.
    pub fn epoch_lock_loss(&self) -> Vec<Epoch> {
        self.lli_and_mask(observation::LliFlags::LOCK_LOSS).epochs()
    }

    /// Removes all observations where receiver phase lock was lost.   
    /// This is only relevant on OBS RINEX.
    pub fn lock_loss_filter_mut(&mut self) {
        self.lli_and_mask_mut(observation::LliFlags::LOCK_LOSS)
    }

    /// Lists all [Constellation] contained in record.    
    /// This applies to OBS, NAV and CLK RINEX.
    pub fn constellations(&self) -> Vec<Constellation> {
        let mut ret: Vec<Constellation> = Vec::new();
        match self.header.constellation {
            Some(Constellation::Mixed) => {
                if let Some(r) = self.record.as_obs() {
                    for (_e, (_clk, vehicles)) in r {
                        for (sv, _) in vehicles {
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
        ret.sort();
        ret
    }

    pub fn retain_best_elevation_angles_mut(&mut self) {
        let best_vehicles = self.space_vehicles_best_elevation_angle();
        if let Some(record) = self.record.as_mut_nav() {
            record.retain(|e, classes| {
                let best = best_vehicles.get(e).unwrap();
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

    /// Returns Sv (embedded) clock biases, in the form
    /// (offset (s), drift (s.s⁻¹), drift rate (s.s⁻²)).   
    /// This only applies to NAV RINEX with Ephemeric frames.
    ///
    /// Example:
    /// ```
    /// use rinex::*;
    /// use std::str::FromStr; // filter!
    /// use rinex::processing::*; // .filter_mut()
    ///
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// rinex.filter_mut(filter!("G08,G07"));
    ///
    /// let sv_clock = rinex.sv_clock();
    /// for (epoch, vehicles) in sv_clock {
    ///     for (sv, (offset, dr, drr)) in vehicles {
    ///         // sv: space vehicle
    ///         // offset [s]
    ///         // dr: clock drift [s.s⁻¹]
    ///         // drr: clock drift rate [s.s⁻²]
    ///     }
    /// }
    /// ```
    pub fn sv_clock(&self) -> BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64)>> {
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

    /// Returns all [Sv] identified in this file.   
    /// For Clocks RINEX: returns list of Vehicles used as reference
    /// in the record.
    /// ```
    /// use rinex::*;
    /// use rinex::prelude::*;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    /// let vehicles = rnx.sv();
    ///
    /// assert_eq!(vehicles, vec![
    ///     sv!("G01"), sv!("G03"), sv!("G06"),
    ///     sv!("G07"), sv!("G08"), sv!("G09"),
    ///     sv!("G11"), sv!("G14"), sv!("G16"),
    ///     sv!("G17"), sv!("G19"), sv!("G22"),
    ///     sv!("G23"), sv!("G26"), sv!("G27"),
    ///     sv!("G28"), sv!("G30"), sv!("G31"),
    ///     sv!("G32")]);
    /// ```
    pub fn sv(&self) -> Vec<Sv> {
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
            for (_, (_, vehicles)) in r {
                for (sv, _) in vehicles {
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

    /// List all [Sv] per epoch of appearance.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    ///
    /// let data = rnx.sv_epoch();
    ///
    /// let first_epoch = Epoch::from_gregorian_utc(2017, 1, 1, 0, 0, 0, 0);
    ///
    /// let vehicles = data.get(&first_epoch)
    ///     .unwrap();
    /// assert_eq!(*vehicles, vec![
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
    pub fn sv_epoch(&self) -> BTreeMap<Epoch, Vec<Sv>> {
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
            for ((epoch, _), (_, vehicles)) in r {
                let mut inner: Vec<Sv> = Vec::new();
                for (sv, _) in vehicles.iter() {
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

    /// List [clocks::record::System] (reference systems) contained in this CLK RINEX.   
    /// Reference systems can either be an Sv or a ground station.
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

    /// List Reference Ground Stations only, used in this CLK RINEX.  
    /// To list reference Sv, simply use [Rinex::sv].
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

    /// Lists identified [navigation::MsgType] in this NAV RINEX.
    pub fn nav_message_types(&self) -> Vec<navigation::MsgType> {
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
    /// This also drops observations that did not come with an LLI flag.  
    /// Only relevant on OBS RINEX.
    pub fn lli_and_mask_mut(&mut self, mask: observation::LliFlags) {
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

    /// [Rinex::lli_and_mask] immutable implementation.   
    /// Only relevant on OBS RINEX.
    pub fn lli_and_mask(&self, mask: observation::LliFlags) -> Self {
        let mut c = self.clone();
        c.lli_and_mask_mut(mask);
        c
    }

    /// Extracts signal strength as (min, max) duplet,
    /// accross all vehicles.   
    /// Only relevant on Observation RINEX.
    pub fn observation_ssi_minmax(&self) -> Option<(observation::Snr, observation::Snr)> {
        let mut ret: Option<(observation::Snr, observation::Snr)> = None;
        if let Some(r) = self.record.as_obs() {
            for (_, (_, vehicles)) in r.iter() {
                for (_, observation) in vehicles.iter() {
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
    /// per vehicle. Only relevant on Observation RINEX
    pub fn observation_ssi_sv_minmax(&self) -> HashMap<Sv, (observation::Snr, observation::Snr)> {
        let mut map: HashMap<Sv, (observation::Snr, observation::Snr)> = HashMap::new();
        if let Some(r) = self.record.as_obs() {
            for (_, (_, vehicles)) in r.iter() {
                for (sv, observations) in vehicles.iter() {
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
    /// Aligns Phase observations at origin
    pub fn observation_phase_align_origin_mut(&mut self) {
        let mut init_phases: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicles)) in r.iter_mut() {
                for (sv, observations) in vehicles.iter_mut() {
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
    /// Aligns Phase observations at origin,
    /// immutable implementation
    pub fn observation_phase_align_origin(&self) -> Self {
        let mut s = self.clone();
        s.observation_phase_align_origin_mut();
        s
    }
    /// Converts all Phase Data to Carrier Cycles by multiplying all phase points
    /// by the carrier signal wavelength.
    pub fn observation_phase_carrier_cycles_mut(&mut self) {
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicles)) in r.iter_mut() {
                for (sv, observations) in vehicles.iter_mut() {
                    for (observable, data) in observations.iter_mut() {
                        if observable.is_phase_observable() {
                            if let Ok(carrier) = observable.carrier(sv.constellation) {
                                data.obs *= carrier.wavelength();
                            }
                        }
                    }
                }
            }
        }
    }

    /// Converts all Phase Data to Carrier Cycles by multiplying all phase points
    /// by the carrier signal wavelength.
    pub fn observation_phase_carrier_cycles(&self) -> Self {
        let mut s = self.clone();
        s.observation_phase_carrier_cycles_mut();
        s
    }

    /// Ionospheric delay detector
    pub fn observation_iono_delay_detector(
        &self,
    ) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> {
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
    /*
        /// Single step /stage, in high order phase differencing
        /// algorithm, which we use in case of old receiver data / old RINEX
        /// to cancel geometric and atmospheric biases.
        /// See [high_order_phase_difference]
        fn high_order_phase_difference_step(&self) -> Result<BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>> {
            let mut ret: BTreeMap<Epoch, HashMap<String, f64>> = BTreeMap::new();
        }

        /// Computes High Order Phase Difference
        /// accross vehicles and epochs,
        /// until differencing order is reached.
        /// This is used in Geometric biases estimation,
        /// in case of single channel receivers / old RINEX, where
        /// only one carrier signal was sampled.
        /// Final order is determined from the epoch interval
        /// (the smallest the better), the phase data quality and so on.
        fn high_order_phase_difference(&self, order: usize) -> Result<BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>> {
            let mut ret: BTreeMap<Epoch, HashMap<String, f64>> = BTreeMap::new();
            if let Some(rec) = self.record.as_obs() {
                for (epoch, (_, vehicles)) in rec {
                    for (sv, observations) in vehicles {
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
        /// on at least two seperate carrier frequencies, for a given space vehicle at a certain epoch.
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
                        // conditions were met for this vehicle
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
        /// from this Observation record, on an epoch basis an per space vehicle.
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
        /// on at least two seperate carrier frequencies, for a given space vehicle at a certain epoch.
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
                        // conditions were met for this vehicle
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
        /// Distant clock offsets can be obtained with [space_vehicles_clock_offset].
        /// Real distances are extracted on an epoch basis, and per space vehicle.
        /// This method has no effect on non observation data.
        ///
        /// Example:
        /// ```
        /// use rinex::prelude::*;
        /// // obtain distance clock offsets, by analyzing a related NAV file
        /// // (this is only an example..)
        /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
        ///     .unwrap();
        /// // Retain G07 + G08 vehicles
        /// // to perform further calculations on these vehicles data (GPS + Svnn filter)
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
        ///     .retain_space_vehicle_mut(filter.clone());
        /// // extract distant clock offsets
        /// let sv_clk_offsets = rinex.space_vehicles_clock_offset();
        /// let rinex = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx");
        /// let mut rinex = rinex.unwrap();
        /// // apply the same filter
        /// rinex
        ///     .retain_space_vehicle_mut(filter.clone());
        /// let distances = rinex.observation_pseudodistances(sv_clk_offsets);
        /// // exploit distances
        /// for (e, sv) in distances.iter() { // (epoch, vehicles)
        ///     for (sv, obs) in sv.iter() { // (vehicle, distance)
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

impl Merge for Rinex {
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

#[cfg(feature = "nav")]
use hifitime::Unit;

#[cfg(feature = "nav")]
use map_3d::ecef2geodetic;

#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
impl Rinex {
    /// Returns Sv position vectors, expressed in meters ECEF for all Epochs.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// let sv_positions = rinex.sv_position();
    /// for (sv, epochs) in sv_positions {
    ///     for (epoch, (sv_x, sv_y, sv_z)) in epochs {
    ///     }
    /// }
    /// ```
    pub fn sv_position(&self) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> {
        let mut ret: HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> = HashMap::new();
        if let Some(record) = self.record.as_nav() {
            for (epoch, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, sv, ephemeris) = fr.as_eph().unwrap();
                            if let Some(sat_pos) = ephemeris.sv_position(*epoch) {
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

    /// Returns Sv position vectors as geodetic WGS84 coordinates
    /// expressed in decimal degrees, for all Epochs.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// let sv_positions = rinex.sv_position_geo();
    /// for (sv, epochs) in sv_positions {
    ///     for (epoch, (sv_lat, sv_lon, sv_alt)) in epochs {
    ///     }
    /// }
    /// ```
    pub fn sv_position_geo(&self) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> {
        let mut data = self.sv_position();
        for (_, epochs) in data.iter_mut() {
            for (_, (sv_x, sv_y, sv_z)) in epochs.iter_mut() {
                let (lat, lon, alt) = ecef2geodetic(*sv_x, *sv_y, *sv_z, map_3d::Ellipsoid::WGS84);
                *sv_x = lat;
                *sv_y = lon;
                *sv_z = alt;
            }
        }
        data
    }

    /// Computes (Elevation, Azim) angles expressed in degrees,
    /// for all Sv through all Epochs, by solving Kepler equations.
    /// A reference ground position must be known:
    ///   - either it is defined in Self
    ///   - otherwise it can be superceeded by user defined position
    /// ```
    /// use rinex::*;
    /// use rinex::prelude::*;
    /// let ref_pos = wgs84!(3582105.291, 532589.7313, 5232754.8054);
    ///
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///     .unwrap();
    ///
    /// let sv_angles = rinex.sv_elev_azim_angles(Some(ref_pos));
    /// for (sv, epochs) in sv_angles {
    ///     for (epoch, (elev, azim)) in epochs {
    ///         // do something
    ///     }
    /// }
    /// ```
    pub fn sv_elev_azim_angles(
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
                            if let Some((el, az)) = ephemeris.sv_elev_azim(ref_pos, *epoch) {
                                if let Some(data) = ret.get_mut(sv) {
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

    /// Returns Sv instantaneous speed vectors, expressed in meters/sec ECEF
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// let sv_speeds = rinex.sv_speed();
    /// for (sv, epochs) in sv_speeds {
    ///     for (epoch, (sv_x, sv_y, sv_z)) in epochs {
    ///     }
    /// }
    /// ```
    pub fn sv_speed(&self) -> HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> {
        let mut ret: HashMap<Sv, BTreeMap<Epoch, (f64, f64, f64)>> = HashMap::new();
        let mut prev_pos: HashMap<Sv, (Epoch, (f64, f64, f64))> = HashMap::new();
        if let Some(record) = self.record.as_nav() {
            for (epoch, classes) in record {
                for (class, frames) in classes {
                    if *class == navigation::FrameClass::Ephemeris {
                        for fr in frames {
                            let (_, sv, ephemeris) = fr.as_eph().unwrap();
                            if let Some(sv_pos) = ephemeris.sv_position(*epoch) {
                                if let Some((prev_epoch, prev_pos)) = prev_pos.get_mut(&sv) {
                                    // compute
                                    let dt = (*epoch - *prev_epoch).to_unit(Unit::Second);
                                    let dx = (sv_pos.0 - prev_pos.0) / dt;
                                    let dy = (sv_pos.1 - prev_pos.1) / dt;
                                    let dz = (sv_pos.2 - prev_pos.2) / dt;
                                    // append
                                    if let Some(data) = ret.get_mut(&sv) {
                                        data.insert(*epoch, (dx, dy, dz));
                                    } else {
                                        let mut map: BTreeMap<Epoch, (f64, f64, f64)> =
                                            BTreeMap::new();
                                        map.insert(*epoch, (dx, dy, dz));
                                        ret.insert(*sv, map);
                                    }

                                    // update
                                    *prev_epoch = *epoch;
                                    *prev_pos = sv_pos;
                                } else {
                                    prev_pos.insert(*sv, (*epoch, sv_pos));
                                }
                            }
                        }
                    }
                }
            }
        }
        ret
    }

    /// Returns list of vehicles per constellation and on an epoch basis
    /// that are closest to Zenith. This is basically a max() operation
    /// on the elevation angle, per epoch and constellation.
    /// This can only be computed on Navigation ephemeris.
    pub fn space_vehicles_best_elevation_angle(&self) -> BTreeMap<Epoch, Vec<Sv>> {
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
                                            if let Some((ddx, ddy, ddz)) = ephemeris.sat_accel_ecef(*epoch, *p_speed, *p_epoch) {

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
}

impl Split for Rinex {
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
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

impl Smooth for Rinex {
    fn moving_average(&self, window: Duration) -> Self {
        let mut s = self.clone();
        s.moving_average_mut(window);
        s
    }
    fn moving_average_mut(&mut self, window: Duration) {
        if let Some(r) = self.record.as_mut_obs() {
            r.moving_average_mut(window);
        }
    }
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

use crate::algorithm::{Filter, Preprocessing};

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

use crate::algorithm::Decimate;

impl Decimate for Rinex {
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        self.record.decimate_by_ratio_mut(r);
    }
    fn decimate_by_interval(&self, dt: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(dt);
        s
    }
    fn decimate_by_interval_mut(&mut self, dt: Duration) {
        self.record.decimate_by_interval_mut(dt);
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        self.record.decimate_match_mut(&rhs.record);
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(rhs);
        s
    }
}

#[cfg(feature = "obs")]
use crate::observation::Observation;

#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
impl Observation for Rinex {
    fn min(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        if let Some(r) = self.record.as_obs() {
            r.min()
        } else {
            (None, HashMap::new())
        }
    }
    fn min_observable(&self) -> HashMap<Observable, f64> {
        if let Some(r) = self.record.as_obs() {
            r.min_observable()
        } else if let Some(r) = self.record.as_meteo() {
            r.min_observable()
        } else {
            HashMap::new()
        }
    }
    fn max(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        if let Some(r) = self.record.as_obs() {
            r.max()
        } else {
            (None, HashMap::new())
        }
    }
    fn max_observable(&self) -> HashMap<Observable, f64> {
        if let Some(r) = self.record.as_obs() {
            r.max_observable()
        } else if let Some(r) = self.record.as_meteo() {
            r.max_observable()
        } else {
            HashMap::new()
        }
    }
    fn mean(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        if let Some(r) = self.record.as_obs() {
            r.mean()
        } else {
            (None, HashMap::new())
        }
    }
    fn mean_observable(&self) -> HashMap<Observable, f64> {
        if let Some(r) = self.record.as_obs() {
            r.mean_observable()
        } else if let Some(r) = self.record.as_meteo() {
            r.mean_observable()
        } else {
            HashMap::new()
        }
    }
    fn std_dev(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        if let Some(r) = self.record.as_obs() {
            r.std_dev()
        } else {
            (None, HashMap::new())
        }
    }
    fn std_dev_observable(&self) -> HashMap<Observable, f64> {
        HashMap::new()
    }
    fn std_var(&self) -> (Option<f64>, HashMap<Sv, HashMap<Observable, f64>>) {
        if let Some(r) = self.record.as_obs() {
            r.std_var()
        } else {
            (None, HashMap::new())
        }
    }
    fn std_var_observable(&self) -> HashMap<Observable, f64> {
        HashMap::new()
    }
}

#[cfg(feature = "obs")]
use observation::Dcb;

#[cfg(feature = "obs")]
impl Dcb for Rinex {
    fn dcb(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.dcb()
        } else {
            panic!("wrong rinex type");
        }
    }
}

#[cfg(feature = "obs")]
use observation::Mp;

#[cfg(feature = "obs")]
impl Mp for Rinex {
    fn mp(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.dcb()
        } else {
            panic!("wrong rinex type");
        }
    }
}

#[cfg(feature = "obs")]
use observation::Combine;

#[cfg(feature = "obs")]
impl Combine for Rinex {
    fn geo_free(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.geo_free()
        } else {
            panic!("wrong RINEX type");
        }
    }
    fn wide_lane(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.wide_lane()
        } else {
            panic!("wrong RINEX type");
        }
    }
    fn narrow_lane(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.narrow_lane()
        } else {
            panic!("wrong RINEX type");
        }
    }
    fn melbourne_wubbena(
        &self,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.melbourne_wubbena()
        } else {
            panic!("wrong RINEX type");
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_macros() {
        assert_eq!(is_comment!("This is a comment COMMENT"), true);
        assert_eq!(is_comment!("This is a comment"), false);
        let _ = sv!("G01");
        let _ = sv!("R03");
        let _ = gnss!("GPS");
        let _ = observable!("L1C");
        let _ = filter!("GPS");
        let _ = filter!("G08, G09");
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
