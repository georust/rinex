#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
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

mod bibliography;
mod ground_position;
mod leap;
mod observable;

#[cfg(test)]
mod tests;

#[macro_use]
mod macros;

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
    pub use crate::types::Type as RinexType;
    pub use crate::Rinex;
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries};
}

#[cfg(feature = "processing")]
mod algorithm;

/// Package to include all preprocessing
/// methods like filtering, data smoothing and masking.
#[cfg(feature = "processing")]
#[cfg_attr(docrs, doc(cfg(feature = "processing")))]
pub mod preprocessing {
    pub use crate::algorithm::*;
}

#[cfg(feature = "qc")]
#[macro_use]
extern crate horrorshow;

use carrier::Carrier;
use prelude::*;

pub use merge::Merge;
pub use split::Split;

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

#[cfg(docrs)]
pub use bibliography::Bibliography;

#[derive(Clone, Default, Debug, PartialEq)]
/// `Rinex` describes a `RINEX` file, it comprises a [Header] section,
/// and a [record::Record] file body.   
/// This parser can also store comments encountered while parsing the file body,
/// stored as [record::Comments], without much application other than presenting
/// all encountered data at the moment.   
/// Following is an example of high level usage (mainly header fields).  
/// For each RINEX type you get a method named after that type, which exposes
/// the whole dataset, for example [`Self::meteo`] for Meteo RINEX.
/// Other (high level information, calculations) are type dependent and
/// contained in a specific crate feature.
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
    HeaderParsingError(#[from] header::ParsingError),
    #[error("record parsing error")]
    RecordError(#[from] record::Error),
    #[error("file i/o error")]
    IoError(#[from] std::io::Error),
}

impl Rinex {
    /// Builds a new `RINEX` struct from given header & body sections.
    pub fn new(header: Header, record: record::Record) -> Rinex {
        Rinex {
            header,
            record,
            comments: record::Comments::new(),
        }
    }

    /// Returns a copy of self with given header attributes.
    pub fn with_header(&self, header: Header) -> Self {
        Rinex {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
        }
    }

    /// Replaces header section.
    pub fn replace_header(&mut self, header: Header) {
        self.header = header.clone();
    }

    /// Returns a copy of self with given internal record.
    pub fn with_record(&self, record: record::Record) -> Self {
        Rinex {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record,
        }
    }

    /// Replaces internal record.
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
    /// assert!(crinex.to_file("test.crx").is_ok(), "failed to generate file");
    /// ```
    pub fn rnx2crnx(&self) -> Self {
        let mut s = self.clone();
        s.rnx2crnx_mut();
        s
    }

    /// [`Self::rnx2crnx`] mutable implementation
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

    /// [`Self::rnx2crnx1`] mutable implementation.
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

    /// [`Self::rnx2crnx3`] mutable implementation.
    pub fn rnx2crnx3_mut(&mut self) {
        if self.is_observation_rinex() {
            self.header = self.header.with_crinex(Crinex {
                date: epoch::now(),
                version: Version { major: 3, minor: 0 },
                prog: "rust-crinex".to_string(),
            });
        }
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
            | types::Type::ClockData => self.epoch().next().unwrap(),
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
        /* This will be required if we ever make the BufferedReader Hatanaka compliant
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
    pub fn is_antex(&self) -> bool {
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
    /// Returns true if Differential Code Biases (DCBs)
    /// are compensated for, in this file, for this GNSS constellation.
    /// DCBs are biases due to tiny frequency differences,
    /// in both the SV embedded code generator, and receiver PLL.
    /// If this is true, that means all code signals received in from
    /// all SV within that constellation, have intrinsinc DCB compensation.
    /// In very high precision and specific applications, you then do not have
    /// to deal with their compensation yourself.
    pub fn dcb_compensation(&self, constellation: Constellation) -> bool {
        self.header
            .dcb_compensations
            .iter()
            .filter(|dcb| dcb.constellation == constellation)
            .count()
            > 0
    }

    /// Returns true if Antenna Phase Center variations are compensated
    /// for in this file. Useful for high precision application.
    pub fn pcv_compensation(&self, constellation: Constellation) -> bool {
        self.header
            .pcv_compensations
            .iter()
            .filter(|pcv| pcv.constellation == constellation)
            .count()
            > 0
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

    //TODO: move to ObsverationIter
    // /// Returns [`Epoch`]s where a loss of lock event happened.
    // /// This is only relevant on OBS RINEX.
    // pub fn epoch_lock_loss(&self) -> Vec<Epoch> {
    //     self.lli_and_mask(observation::LliFlags::LOCK_LOSS).epoch()
    // }

    /// Removes all observations where receiver phase lock was lost.   
    /// This is only relevant on OBS RINEX.
    pub fn lock_loss_filter_mut(&mut self) {
        self.lli_and_mask_mut(observation::LliFlags::LOCK_LOSS)
    }

    pub fn retain_best_elevation_angles_mut(&mut self) {
        unimplemented!("retain_best_elev: use preprocessing toolkit instead");
        //let best_vehicles = self.space_vehicles_best_elevation_angle();
        //if let Some(record) = self.record.as_mut_nav() {
        //    record.retain(|e, classes| {
        //        let best = best_vehicles.get(e).unwrap();
        //        classes.retain(|class, frames| {
        //            if *class == navigation::FrameClass::Ephemeris {
        //                frames.retain(|fr| {
        //                    let (_, sv, _) = fr.as_eph().unwrap();
        //                    best.contains(sv)
        //                });
        //                frames.len() > 0
        //            } else {
        //                false
        //            }
        //        });
        //        classes.len() > 0
        //    });
        //}
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
                        if let clocks::System::Station(station) = system {
                            if !ret.contains(station) {
                                ret.push(station.clone());
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

    /// [`Rinex::lli_and_mask`] immutable implementation.   
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
    /// Aligns Phase observations at origin
    pub fn observation_phase_align_origin_mut(&mut self) {
        let mut init_phases: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
        if let Some(r) = self.record.as_mut_obs() {
            for (_, (_, vehicles)) in r.iter_mut() {
                for (sv, observations) in vehicles.iter_mut() {
                    for (observable, data) in observations.iter_mut() {
                        if observable.is_phase_observable() {
                            if let Some(init_phase) = init_phases.get_mut(sv) {
                                if init_phase.get(observable).is_none() {
                                    init_phase.insert(observable.clone(), data.obs);
                                }
                            } else {
                                let mut map: HashMap<Observable, f64> = HashMap::new();
                                map.insert(observable.clone(), data.obs);
                                init_phases.insert(*sv, map);
                            }
                            data.obs -= init_phases.get(sv).unwrap().get(observable).unwrap();
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
                !systems.is_empty()
            });
            !dtypes.is_empty()
        })
    }
    /// Writes self into given file.   
    /// Both header + record will strictly follow RINEX standards.   
    /// Record: refer to supported RINEX types
    pub fn to_file(&self, path: &str) -> Result<(), Error> {
        let mut writer = BufferedWriter::new(path)?;
        write!(writer, "{}", self.header)?;
        self.record.to_file(&self.header, &mut writer)?;
        Ok(())
    }
}

/*
 * Sampling related methods
 */
impl Rinex {
    /// Returns first [`Epoch`] encountered in time
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch().next()
    }

    /// Returns last [`Epoch`] encountered in time
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch().last()
    }

    /// Returns Duration of (time spanned by) this RINEX
    pub fn duration(&self) -> Option<Duration> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        Some(end - start)
    }

    /// Form a [`Timeseries`] iterator spanning [Self::duration]
    /// with [Self::dominant_sample_rate] spacing
    pub fn timeseries(&self) -> Option<TimeSeries> {
        let start = self.first_epoch()?;
        let end = self.last_epoch()?;
        let dt = self.dominant_sample_rate()?;
        Some(TimeSeries::inclusive(start, end, dt))
    }

    /// Returns sample rate used by the data receiver.
    pub fn sample_rate(&self) -> Option<Duration> {
        self.header.sampling_interval
    }

    /// Returns dominant sample rate
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// assert_eq!(
    ///     rnx.dominant_sample_rate(),
    ///     Some(Duration::from_seconds(60.0)));
    /// ```
    pub fn dominant_sample_rate(&self) -> Option<Duration> {
        self.sampling_histogram()
            .max_by(|(_, x_pop), (_, y_pop)| x_pop.cmp(y_pop))
            .map(|dominant| dominant.0)
    }

    /// ```
    /// use rinex::prelude::*;
    /// use itertools::Itertools;
    /// use std::collections::HashMap;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    ///  assert!(
    ///     rinex.sampling_histogram().sorted().eq(vec![
    ///         (Duration::from_seconds(15.0 * 60.0), 1),
    ///         (Duration::from_seconds(25.0 * 60.0), 1),
    ///         (Duration::from_seconds(4.0 * 3600.0 + 45.0 * 60.0), 2),
    ///         (Duration::from_seconds(5.0 * 3600.0 + 30.0 * 60.0), 1),
    ///     ]),
    ///     "sampling_histogram failed"
    /// );
    /// ```
    pub fn sampling_histogram(&self) -> Box<dyn Iterator<Item = (Duration, usize)> + '_> {
        // compute dt = |e_k+1 - e_k| : instantaneous epoch delta
        //              then compute an histogram on these intervals
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .map(|(ek, ekp1)| ekp1 - ek) // following step computes the histogram
                // and at the same time performs a .unique() like filter
                .fold(vec![], |mut list, dt| {
                    let mut found = false;
                    for (delta, pop) in list.iter_mut() {
                        if *delta == dt {
                            *pop += 1;
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        list.push((dt, 1));
                    }
                    list
                })
                .into_iter(),
        )
    }

    /// Returns an iterator over unexpected data gaps,
    /// in the form ([`Epoch`], [`Duration`]), where
    /// epoch is the starting datetime, and its related duration.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::prelude::{Rinex, Epoch, Duration};
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    ///
    /// // when tolerance is set to None,
    /// // the reference sample rate is [Self::dominant_sample_rate].
    /// let mut tolerance : Option<Duration> = None;
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(None).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:02:00 UTC").unwrap(), Duration::from_seconds(7.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T23:21:00 UTC").unwrap(), Duration::from_seconds(31.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=None) failed"
    /// );
    ///
    /// // with a tolerance, we tolerate the given gap duration
    /// tolerance = Some(Duration::from_seconds(3600.0));
    /// let gaps : Vec<_> = rinex.data_gaps(tolerance).collect();
    /// assert!(
    ///     rinex.data_gaps(Some(Duration::from_seconds(3.0 * 3600.0))).eq(
    ///         vec![
    ///             (Epoch::from_str("2015-01-01T00:09:00 UTC").unwrap(), Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T09:04:00 UTC").unwrap(), Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)),
    ///             (Epoch::from_str("2015-01-01T19:54:00 UTC").unwrap(), Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)),
    ///         ]),
    ///     "data_gaps(tol=3h) failed"
    /// );
    /// ```
    pub fn data_gaps(
        &self,
        tolerance: Option<Duration>,
    ) -> Box<dyn Iterator<Item = (Epoch, Duration)> + '_> {
        let sample_rate: Duration = match tolerance {
            Some(dt) => dt, // user defined
            None => {
                match self.dominant_sample_rate() {
                    Some(dt) => dt,
                    None => {
                        match self.sample_rate() {
                            Some(dt) => dt,
                            None => {
                                // not enough information
                                // this is probably not an Epoch iterated RINEX
                                return Box::new(Vec::<(Epoch, Duration)>::new().into_iter());
                            },
                        }
                    },
                }
            },
        };
        Box::new(
            self.epoch()
                .zip(self.epoch().skip(1))
                .filter_map(move |(ek, ekp1)| {
                    let dt = ekp1 - ek; // gap
                    if dt > sample_rate {
                        // too large
                        Some((ek, dt)) // retain starting datetime and gap duration
                    } else {
                        None
                    }
                }),
        )
    }
}

/*
 * Methods that return an Iterator exclusively.
 * These methods are used to browse data easily and efficiently.
 * It includes Format dependent extraction methods : one per format.
 */
use itertools::Itertools; // .unique()
use observation::ObservationData;

impl Rinex {
    pub fn epoch(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        if let Some(r) = self.record.as_obs() {
            Box::new(r.iter().map(|((k, _), _)| *k))
        } else if let Some(r) = self.record.as_nav() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_meteo() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_clock() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else if let Some(r) = self.record.as_ionex() {
            Box::new(r.iter().map(|(k, _)| *k))
        } else {
            panic!(
                "cannot get an epoch iterator for \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }

    /// Returns a unique [`Sv`] iterator, to navigate
    /// all Satellite Vehicles encountered and identified.
    /// This will panic if invoked on ATX, Meteo or IONEX records.
    /// In case of Clock RINEX, the returns the list of vehicles
    /// used as reference.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::sv; // sv!
    /// use std::str::FromStr; // sv!
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    /// let mut vehicles : Vec<_> = rnx.sv().collect(); // to run comparison
    /// vehicles.sort(); // to run comparison
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
    pub fn sv(&self) -> Box<dyn Iterator<Item = Sv> + '_> {
        if let Some(record) = self.record.as_obs() {
            Box::new(
                // grab all vehicles identified through all Epochs
                // and fold them into a unique list
                record
                    .iter()
                    .map(|((_, _), (_clk, entries))| {
                        let sv: Vec<Sv> = entries.keys().cloned().collect();
                        sv
                    })
                    .fold(vec![], |mut list, new_items| {
                        for new in new_items {
                            if !list.contains(&new) {
                                // create a unique list
                                list.push(new);
                            }
                        }
                        list
                    })
                    .into_iter(),
            )
        } else if let Some(record) = self.record.as_nav() {
            Box::new(
                // grab all vehicles through all epochs,
                // fold them into a unique list
                record
                    .into_iter()
                    .flat_map(|(_, frames)| {
                        frames
                            .iter()
                            .filter_map(|fr| {
                                if let Some((_, sv, _)) = fr.as_eph() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_eop() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_ion() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_sto() {
                                    Some(*sv)
                                } else {
                                    None
                                }
                            })
                            .fold(vec![], |mut list, sv| {
                                list.push(sv);
                                list
                            })
                            .into_iter()
                    })
                    .unique(), /*.fold(vec![], |mut list, sv| {
                                   if !list.contains(&sv) {
                                       list.push(sv);
                                   }
                                   list
                               })
                               .into_iter(),
                               */
            )
        } else {
            panic!(
                ".sv() is not feasible on \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
        //} else if let Some(r) = self.record.as_clock() {
        //    for (_, dtypes) in r {
        //        for (_, systems) in dtypes {
        //            for (system, _) in systems {
        //                match system {
        //                    clocks::System::Sv(sv) => {
        //                        if !map.contains(sv) {
        //                            map.push(*sv);
        //                        }
        //                    },
        //                    _ => {},
        //                }
        //            }
        //        }
        //    }
        //}
        //map.sort();
        //map
    }

    /// List all [`Sv`] per epoch of appearance.
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
    ///     .unwrap();
    ///
    /// let mut data = rnx.sv_epoch();
    ///
    /// if let Some((epoch, vehicles)) = data.nth(0) {
    ///     assert_eq!(epoch,Epoch::from_gregorian_utc(2017, 1, 1, 0, 0, 0, 0));
    ///     let expected = vec![
    ///         Sv::new(Constellation::GPS, 03),
    ///         Sv::new(Constellation::GPS, 08),
    ///         Sv::new(Constellation::GPS, 14),
    ///         Sv::new(Constellation::GPS, 16),
    ///         Sv::new(Constellation::GPS, 22),
    ///         Sv::new(Constellation::GPS, 23),
    ///         Sv::new(Constellation::GPS, 26),
    ///         Sv::new(Constellation::GPS, 27),
    ///         Sv::new(Constellation::GPS, 31),
    ///         Sv::new(Constellation::GPS, 32),
    ///     ];
    ///     assert_eq!(*vehicles, expected);
    /// }
    /// ```
    pub fn sv_epoch(&self) -> Box<dyn Iterator<Item = (Epoch, Vec<Sv>)> + '_> {
        if let Some(record) = self.record.as_obs() {
            Box::new(
                // grab all vehicles identified through all Epochs
                // and fold them into individual lists
                record.iter().map(|((epoch, _), (_clk, entries))| {
                    (*epoch, entries.keys().unique().cloned().collect())
                }),
            )
        } else if let Some(record) = self.record.as_nav() {
            Box::new(
                // grab all vehicles through all epochs,
                // fold them into individual lists
                record.iter().map(|(epoch, frames)| {
                    (
                        *epoch,
                        frames
                            .iter()
                            .filter_map(|fr| {
                                if let Some((_, sv, _)) = fr.as_eph() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_eop() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_ion() {
                                    Some(*sv)
                                } else if let Some((_, sv, _)) = fr.as_sto() {
                                    Some(*sv)
                                } else {
                                    None
                                }
                            })
                            .fold(vec![], |mut list, sv| {
                                if !list.contains(&sv) {
                                    list.push(sv);
                                }
                                list
                            }),
                    )
                }),
            )
        } else {
            panic!(
                ".sv_epoch() is not feasible on \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }
    /// Returns a (unique) Iterator over all identified [`Constellation`]s.
    /// ```
    /// use rinex::prelude::*;
    /// use itertools::Itertools; // .sorted()
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx")
    ///     .unwrap();
    ///
    /// assert!(
    ///     rnx.constellation().sorted().eq(
    ///         vec![
    ///             Constellation::GPS,
    ///             Constellation::Glonass,
    ///             Constellation::BeiDou,
    ///             Constellation::Galileo,
    ///         ]
    ///     ),
    ///     "parsed wrong GNSS context",
    /// );
    /// ```
    pub fn constellation(&self) -> Box<dyn Iterator<Item = Constellation> + '_> {
        // from .sv() (unique) iterator:
        //  create a unique list of Constellations
        Box::new(self.sv().map(|sv| sv.constellation).unique())
    }
    /// Returns an Iterator over Unique Constellations, per Epoch
    pub fn constellation_epoch(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, Vec<Constellation>)> + '_> {
        Box::new(self.sv_epoch().map(|(epoch, svnn)| {
            (
                epoch,
                svnn.iter().map(|sv| sv.constellation).unique().collect(),
            )
        }))
    }
    /// Returns a (unique) Iterator over all identified [`Observable`]s.
    /// This will panic if invoked on other than OBS and Meteo RINEX.
    pub fn observable(&self) -> Box<dyn Iterator<Item = &Observable> + '_> {
        if self.record.as_obs().is_some() {
            Box::new(
                self.observation()
                    .map(|(_, (_, svnn))| {
                        svnn.iter()
                            .flat_map(|(_sv, observables)| observables.keys())
                    })
                    .fold(vec![], |mut list, items| {
                        // create a unique list
                        for item in items {
                            if !list.contains(&item) {
                                list.push(item);
                            }
                        }
                        list
                    })
                    .into_iter(),
            )
        } else if self.record.as_meteo().is_some() {
            Box::new(
                self.meteo()
                    .map(|(_, observables)| {
                        observables.keys()
                        //.copied()
                    })
                    .fold(vec![], |mut list, items| {
                        // create a unique list
                        for item in items {
                            if !list.contains(&item) {
                                list.push(item);
                            }
                        }
                        list
                    })
                    .into_iter(),
            )
        } else {
            panic!(
                ".observable() is not feasible on \"{:?}\" RINEX",
                self.header.rinex_type
            );
        }
    }
    /// Meteo RINEX record browsing method. Extracts data for this specific format.
    /// Data is sorted by [`Epoch`] then by [`Observable`].
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///    .unwrap();
    /// for (epoch, observables) in rnx.meteo() {
    ///     println!(" *** Epoch:  {} ****", epoch);
    ///     for (observable, data) in observables {
    ///         println!("{} : {}", observable, data);
    ///     }
    /// }
    /// ```
    pub fn meteo(&self) -> Box<dyn Iterator<Item = (&Epoch, &HashMap<Observable, f64>)> + '_> {
        Box::new(
            self.record
                .as_meteo()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns Observation record iterator. Unlike other records,
    /// an [`EpochFlag`] is attached to each individual [`Epoch`]
    /// to either validated or invalidate it.
    /// Clock receiver offset (in seconds), if present, are defined for each individual
    /// [`Epoch`].
    /// Phase data is exposed as raw / unscaled data: therefore incorrect
    /// values in case of High Precision RINEX. Prefer the dedicated
    /// [Self::carrier_phase] iterator. In any case, you should always
    /// prefer the iteration method of the type of data you're interested in.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::{observable, sv}; // macros
    /// use std::str::FromStr; // observable!, sv!
    ///
    /// let rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx")
    ///    .unwrap();
    ///
    /// for ((epoch, flag), (clock_offset, vehicles)) in rnx.observation() {
    ///     assert!(flag.is_ok()); // no invalid epochs in this file
    ///     assert!(clock_offset.is_none()); // we don't have an example for this, at the moment
    ///     for (sv, observations) in vehicles {
    ///         if *sv == sv!("E01") {
    ///             for (observable, observation) in observations {
    ///                 if *observable == observable!("L1C") {
    ///                     if let Some(lli) = observation.lli {
    ///                         // A flag might be attached to each observation.
    ///                         // Implemented as `bitflag`, it supports bit masking operations
    ///                     }
    ///                     if let Some(snri) = observation.snr {
    ///                         // SNR indicator might exist too
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn observation(
        &self,
    ) -> Box<
        dyn Iterator<
                Item = (
                    &(Epoch, EpochFlag),
                    &(
                        Option<f64>,
                        BTreeMap<Sv, HashMap<Observable, ObservationData>>,
                    ),
                ),
            > + '_,
    > {
        Box::new(
            self.record
                .as_obs()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
}

#[cfg(feature = "obs")]
use crate::observation::Snr;

/*
 * OBS RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "obs")]
#[cfg_attr(docrs, doc(cfg(feature = "obs")))]
impl Rinex {
    /// Returns a Unique Iterator over identified [`Carrier`]s
    pub fn carrier(&self) -> Box<dyn Iterator<Item = Carrier> + '_> {
        Box::new(self.observation().flat_map(|(_, (_, sv))| {
            sv.iter().flat_map(|(sv, observations)| {
                observations
                    .keys()
                    .filter_map(|observable| {
                        if let Ok(carrier) = observable.carrier(sv.constellation) {
                            Some(carrier)
                        } else {
                            None
                        }
                    })
                    .fold(vec![], |mut list, item| {
                        if !list.contains(&item) {
                            list.push(item);
                        }
                        list
                    })
                    .into_iter()
            })
        }))
    }
    pub fn code(&self) -> Box<dyn Iterator<Item = String> + '_> {
        Box::new(
            self.observation()
                .flat_map(|(_, (_, sv))| {
                    sv.iter().flat_map(|(_, observations)| {
                        observations
                            .keys()
                            .filter_map(|observable| observable.code())
                    })
                })
                .unique(),
        )
    }
    /// Returns ([`Epoch`] [`EpochFlag`]) iterator, where each {`EpochFlag`]
    /// validates or invalidates related [`Epoch`]
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for (epoch, flag) in rnx.epoch_flag() {
    ///     assert!(flag.is_ok()); // no invalid epoch
    /// }
    /// ```
    pub fn epoch_flag(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.observation().map(|(e, _)| *e))
    }
    /// Returns an Iterator over all abnormal [`Epoch`]s
    /// and reports given event nature.  
    /// Refer to [`epoch::EpochFlag`] for all possible events.  
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_anomalies(&self) -> Box<dyn Iterator<Item = (Epoch, EpochFlag)> + '_> {
        Box::new(self.epoch_flag().filter_map(
            |(e, f)| {
                if !f.is_ok() {
                    Some((e, f))
                } else {
                    None
                }
            },
        ))
    }
    /// Returns an iterator over all [`Epoch`]s that have
    /// an [`EpochFlag::Ok`] flag attached to them
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_ok(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(
            self.epoch_flag()
                .filter_map(|(e, f)| if f.is_ok() { Some(e) } else { None }),
        )
    }
    /// Returns an iterator over all [`Epoch`]s where
    /// a Cycle Slip is declared by the receiver
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// ```
    pub fn epoch_cs(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.epoch_flag().filter_map(|(e, f)| {
            if f == EpochFlag::CycleSlip {
                Some(e)
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over receiver clock offsets, expressed in seconds.
    /// Such information is kind of rare (modern / dual frequency receivers?)
    /// and we don't have a compelling example yet.
    /// ```
    /// use rinex::prelude::Rinex;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// for ((epoch, flag), clk) in rnx.recvr_clock() {
    ///     // epoch: [hifitime::Epoch]
    ///     // clk: receiver clock offset [s]
    /// }
    /// ```
    pub fn recvr_clock(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), f64)> + '_> {
        Box::new(
            self.observation()
                .filter_map(|(e, (clk, _))| clk.as_ref().map(|clk| (*e, *clk))),
        )
    }
    /// Returns an iterator over phase data, expressed in (whole) carrier cycles.
    /// If Self is a High Precision RINEX (scaled RINEX), data is correctly scaled.
    /// High precision RINEX allows up to 100 pico carrier cycle precision.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal iterator
    /// let phase_l1c = rnx.carrier_phase()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("L1C") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn carrier_phase(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(observable, obsdata)| {
                    if observable.is_phase_observable() {
                        if let Some(header) = &self.header.obs {
                            // apply a scaling, if any, otherwise : leave data untouched
                            // to preserve its precision
                            if let Some(scaling) =
                                header.scaling(sv.constellation, observable.clone())
                            {
                                Some((*e, *sv, observable, obsdata.obs / *scaling as f64))
                            } else {
                                Some((*e, *sv, observable, obsdata.obs))
                            }
                        } else {
                            Some((*e, *sv, observable, obsdata.obs))
                        }
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over pseudo range observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a C1 pseudo range iterator
    /// let c1 = rnx.pseudo_range()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("C1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn pseudo_range(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_pseudorange_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over fractional pseudo range observations
    pub fn pseudo_range_fract(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, f64)> + '_> {
        Box::new(self.pseudo_range().filter_map(|(e, sv, observable, pr)| {
            if let Some(t) = observable.code_length(sv.constellation) {
                let c = 299792458_f64; // speed of light
                Some((e, sv, observable, pr / c / t))
            } else {
                None
            }
        }))
    }
    /// Returns an iterator over doppler shifts. A positive doppler
    /// means Sv is moving towards receiver.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a L1 signal doppler iterator
    /// let doppler_l1 = rnx.doppler()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("D1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn doppler(
        &self,
    ) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_doppler_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an iterator over signal strength observations.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observable;
    /// use std::str::FromStr;
    ///
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O")
    ///     .unwrap();
    /// // example: design a S1: L1 strength iterator
    /// let ssi_l1 = rnx.ssi()
    ///     .filter_map(|(e, sv, obs, value)| {
    ///         if *obs == observable!("S1") {
    ///             Some((e, sv, value))
    ///         } else {
    ///             None
    ///         }
    ///     });
    /// ```
    pub fn ssi(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, f64)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations.iter().filter_map(|(obs, obsdata)| {
                    if obs.is_ssi_observable() {
                        Some((*e, *sv, obs, obsdata.obs))
                    } else {
                        None
                    }
                })
            })
        }))
    }
    /// Returns an Iterator over signal SNR indications.
    /// All observation that did not come with such indication are filtered out.
    pub fn snr(&self) -> Box<dyn Iterator<Item = ((Epoch, EpochFlag), Sv, &Observable, Snr)> + '_> {
        Box::new(self.observation().flat_map(|(e, (_, vehicles))| {
            vehicles.iter().flat_map(|(sv, observations)| {
                observations
                    .iter()
                    .filter_map(|(obs, obsdata)| obsdata.snr.map(|snr| (*e, *sv, obs, snr)))
            })
        }))
    }
    /// Returns an Iterator over "complete" Epochs.
    /// "Complete" Epochs are Epochs were both Phase and Pseudo Range
    /// observations are present on two carriers, sane sampling conditions are met
    /// and an optional minimal SNR criteria is met (disregarded if None).
    pub fn complete_epoch(
        &self,
        min_snr: Option<Snr>,
    ) -> Box<dyn Iterator<Item = (Epoch, Vec<(Sv, Carrier)>)> + '_> {
        Box::new(
            self.observation()
                .filter_map(|((e, flag), (_, vehicles))| {
                    if flag.is_ok() {
                        let mut list: Vec<(Sv, Carrier)> = Vec::new();
                        for (sv, observables) in vehicles {
                            let mut l1_pr_ph = (false, false);
                            let mut lx_pr_ph: HashMap<Carrier, (bool, bool)> = HashMap::new();
                            let mut criteria_met = true;
                            for (observable, observation) in observables {
                                if !observable.is_phase_observable()
                                    && !observable.is_pseudorange_observable()
                                {
                                    continue; // not interesting here
                                }
                                let carrier_code = &observable.to_string()[1..2];
                                let carrier =
                                    Carrier::from_observable(sv.constellation, observable)
                                        .unwrap_or(Carrier::default());
                                if carrier == Carrier::L1 {
                                    l1_pr_ph.0 |= observable.is_pseudorange_observable();
                                    l1_pr_ph.1 |= observable.is_phase_observable();
                                } else {
                                    if let Some((lx_pr, lx_ph)) = lx_pr_ph.get_mut(&carrier) {
                                        *lx_pr |= observable.is_pseudorange_observable();
                                        *lx_ph |= observable.is_phase_observable();
                                    } else {
                                        if observable.is_pseudorange_observable() {
                                            lx_pr_ph.insert(carrier, (true, false));
                                        } else if observable.is_phase_observable() {
                                            lx_pr_ph.insert(carrier, (false, true));
                                        }
                                    }
                                }
                            }
                            if l1_pr_ph == (true, true) {
                                for (carrier, (pr, ph)) in lx_pr_ph {
                                    if pr == true && ph == true {
                                        list.push((*sv, carrier));
                                    }
                                }
                            }
                        }
                        Some((*e, list))
                    } else {
                        None
                    }
                })
                .filter(|(sv, list)| !list.is_empty()),
        )
    }
}

#[cfg(feature = "nav")]
use crate::navigation::{
    BdModel, EopMessage, Ephemeris, IonMessage, KbModel, NavFrame, NavMsgType, NgModel, StoMessage,
};

//#[cfg(feature = "nav")]
//use hifitime::Unit;
//.sv_speed()

#[cfg(feature = "nav")]
use map_3d::ecef2geodetic;

/*
 * NAV RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
impl Rinex {
    /// Returns NAV frames interator (any types).
    /// NAV record may contain several different types of frames.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// for (epoch, nav_frames) in rinex.navigation() {
    ///     for frame in nav_frames {
    ///         // this record only contains ephemeris frames
    ///         assert!(frame.as_eph().is_some());
    ///         assert!(frame.as_ion().is_none());
    ///         assert!(frame.as_eop().is_none());
    ///         assert!(frame.as_sto().is_none());
    ///         if let Some((msg, sv, data)) = frame.as_eph() {
    ///             // this record only contains legacy frames
    ///             assert_eq!(msg, NavMsgType::LNAV);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn navigation(&self) -> Box<dyn Iterator<Item = (&Epoch, &Vec<NavFrame>)> + '_> {
        Box::new(
            self.record
                .as_nav()
                .into_iter()
                .flat_map(|record| record.iter()),
        )
    }
    /// Returns a Unique Iterator over [`NavMsgType`]s that were identified
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// assert!(
    ///     rinex.nav_msg_type().eq(
    ///         vec![NavMsgType::LNAV],
    ///     ),
    ///     "this file only contains legacy frames"
    /// );
    /// ```
    pub fn nav_msg_type(&self) -> Box<dyn Iterator<Item = NavMsgType> + '_> {
        Box::new(
            self.navigation()
                .map(|(_, frames)| {
                    frames
                        .into_iter()
                        .filter_map(|fr| {
                            if let Some((msg, _, _)) = fr.as_eph() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_ion() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_eop() {
                                Some(msg)
                            } else if let Some((msg, _, _)) = fr.as_sto() {
                                Some(msg)
                            } else {
                                None
                            }
                        })
                        .fold(vec![], |mut list, msg| {
                            list.push(msg);
                            list
                        })
                        .into_iter()
                })
                .fold(vec![], |mut list, items| {
                    for item in items {
                        if !list.contains(&item) {
                            list.push(item); // create a unique list
                        }
                    }
                    list
                })
                .into_iter(),
        )
    }
    /// Returns Ephemeris frames interator.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::NavMsgType;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// for (epoch, (msg, sv, data)) in rinex.ephemeris() {
    ///     // this record only contains Legacy NAV frames
    ///     assert_eq!(msg, NavMsgType::LNAV);
    /// }
    /// ```
    pub fn ephemeris(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, &Sv, &Ephemeris))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, eph)) = fr.as_eph() {
                    Some((e, (msg, sv, eph)))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns an Iterator over Sv (embedded) clock offset (s), drift (s.s⁻¹) and
    /// drift rate (s.s⁻²)
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx")
    ///     .unwrap();
    /// for (epoch, sv, (offset, drift, drift_rate)) in rinex.sv_clock() {
    ///     // sv: satellite vehicle
    ///     // offset [s]
    ///     // clock drift [s.s⁻¹]
    ///     // clock drift rate [s.s⁻²]
    /// }
    /// ```
    pub fn sv_clock(&self) -> Box<dyn Iterator<Item = (Epoch, Sv, (f64, f64, f64))> + '_> {
        Box::new(
            self.ephemeris()
                .map(|(e, (_, sv, data))| (*e, *sv, data.sv_clock())),
        )
    }
    /// Returns Ephemeris Reference Epoch (a.k.a toe) for desired SV and specified Epoch `t`.
    /// Toe is the central Epoch of broadcasted ephemeris around that instant.
    pub fn sv_toe(&self, sv: Sv, t: Epoch) -> Option<Epoch> {
        let (_, (_, _, ephemeris)) = self
            .ephemeris()
            .find(|(epoch, (_, svnn, _))| **epoch == t && **svnn == sv)?;
        let ts = sv.constellation.timescale()?;
        ephemeris.toe(ts)
    }
    /// Returns an Iterator over Sv (embedded) time offsets between the SV local onboard clock,
    /// and its associated GNSS time scale. Offset expressed as a [`Duration`].
    pub fn sv_clock_offset(&self) -> Box<dyn Iterator<Item = (Epoch, Sv, Duration)> + '_> {
        Box::new(self.sv_clock().filter_map(|(t, sv, (a0, a1, a2))| {
            if let Some(toe) = self.sv_toe(sv, t) {
                let dt = (t - toe).to_seconds();
                let dt_sat = a0 + a1 * dt + a2 * dt.powi(2);
                Some((t, sv, Duration::from_seconds(dt_sat)))
            } else {
                None
            }
        }))
    }
    /// Interpolates SV clock offset @ desired t
    pub fn sv_clock_offset_interpolate(&self, sv: Sv, t: Epoch) -> Option<Duration> {
        //TODO
        self.sv_clock_offset()
            .filter_map(
                |(e, svnn, dt)| {
                    if e == t && svnn == sv {
                        Some(dt)
                    } else {
                        None
                    }
                },
            )
            .reduce(|dt, _| dt) //unique
    }
    /// Returns an Iterator over Sv position vectors,
    /// expressed in km ECEF for all Epochs.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// for (epoch, sv, (x, y, z)) in rinex.sv_position() {
    ///     // sv: satellite vehicle
    ///     // x: x(t) [km ECEF]
    ///     // y: y(t) [km ECEF]
    ///     // z: z(t) [km ECEF]
    /// }
    /// ```
    pub fn sv_position(&self) -> Box<dyn Iterator<Item = (Epoch, Sv, (f64, f64, f64))> + '_> {
        Box::new(self.ephemeris().filter_map(|(e, (_, sv, ephemeris))| {
            if let Some((x, y, z)) = ephemeris.sv_position(sv, *e) {
                Some((*e, *sv, (x, y, z)))
            } else {
                // non feasible calculations.
                // most likely due to missing Keplerian parameters,
                // at this Epoch
                None
            }
        }))
    }
    /// Interpolates SV position, expressed in meters ECEF at desired Epoch `t`.
    /// An interpolation order of at least 7 is recommended.
    /// Operation is not feasible if sampling interval cannot be determined.
    /// In ideal scenarios, Broadcast Ephemeris are complete and evenly spaced in time:
    ///   - the first Epoch we an interpolate is ](N +1)/2 * τ; ...]
    ///   - the last Epoch we an interpolate is  [..;  T - (N +1)/2 * τ]
    /// where N is the interpolation order, τ the broadcast interval and T
    /// the last broadcast message received.
    /// This method is designed to minimize interpolation errors at the expense
    /// of interpolatable Epochs. See [Bibliography::Japhet2021].
    pub fn sv_position_interpolate(
        &self,
        sv: Sv,
        t: Epoch,
        order: usize,
    ) -> Option<(f64, f64, f64)> {
        let odd_order = order % 2 > 0;
        let dt = match self.sample_rate() {
            Some(dt) => dt,
            None => match self.dominant_sample_rate() {
                Some(dt) => dt,
                None => {
                    /*
                     * Can't determine anything: not enough information
                     */
                    return None;
                },
            },
        };

        let sv_position: Vec<_> = self
            .sv_position()
            .filter_map(|(e, svnn, (x, y, z))| {
                if sv == svnn {
                    Some((e, (x, y, z)))
                } else {
                    None
                }
            })
            .collect();
        /*
         * Determine cloesest Epoch in time
         */
        let center = match sv_position.iter().find(|(e, _)| (*e - t).abs() < dt) {
            Some(center) => center,
            None => {
                /*
                 * Failed to determine central Epoch for this SV
                 * empty data set: should not happen
                 */
                return None;
            },
        };
        // println!("CENTRAL EPOCH: {:?}", center); // DEBUG
        let center_pos = match sv_position.iter().position(|(e, _)| *e == center.0) {
            Some(center) => center,
            None => {
                /* will never happen at this point */
                return None;
            },
        };

        let (min_before, min_after): (usize, usize) = match odd_order {
            true => ((order + 1) / 2, (order + 1) / 2),
            false => (order / 2, order / 2 + 1),
        };

        if center_pos < min_before || sv_position.len() - center_pos < min_after {
            /* can't design time window */
            return None;
        }

        let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
        let offset = center_pos - min_before;

        for i in 0..order + 1 {
            let mut li = 1.0_f64;
            let (e_i, (x_i, y_i, z_i)) = sv_position[offset + i];
            for j in 0..order + 1 {
                let (e_j, _) = sv_position[offset + j];
                if j != i {
                    li *= (t - e_j).to_seconds();
                    li /= (e_i - e_j).to_seconds();
                }
            }
            polynomials.0 += x_i * li;
            polynomials.1 += y_i * li;
            polynomials.2 += z_i * li;
        }

        Some(polynomials)
    }
    /// Returns an Iterator over Sv position vectors,
    /// expressed as geodetic coordinates, with latitude and longitude
    /// in decimal degrees.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// for (epoch, sv, (lat, lon, alt)) in rinex.sv_position_geo() {
    ///     // sv: satellite vehicle
    ///     // lat [ddeg]
    ///     // lon [ddeg]
    ///     // alt: [m ECEF]
    /// }
    /// ```
    pub fn sv_position_geo(&self) -> Box<dyn Iterator<Item = (Epoch, Sv, (f64, f64, f64))> + '_> {
        Box::new(self.sv_position().map(|(e, sv, (x, y, z))| {
            let (lat, lon, alt) = ecef2geodetic(x, y, z, map_3d::Ellipsoid::WGS84);
            (e, sv, (lat, lon, alt))
        }))
    }
    /// Returns Iterator over Sv speed vectors, expressed in km/s ECEF.
    /// ```
    /// use rinex::prelude::*;
    ///
    /// let mut rinex =
    ///     Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///         .unwrap();
    ///
    /// //for (epoch, (sv, sv_x, sv_y, sv_z)) in rinex.sv_speed() {
    /// //    // sv_x : km/s
    /// //    // sv_y : km/s
    /// //    // sv_z : km/s
    /// //}
    /// ```
    pub fn sv_speed(&self) -> Box<dyn Iterator<Item = (Epoch, Sv, (f64, f64, f64))> + '_> {
        todo!("sv_speed");
        //Box::new(
        //    self.sv_position()
        //    self.sv_position()
        //        .skip(1)
        //)
    }
    /// Returns an Iterator over Sv elevation and azimuth angles,
    /// both expressed in degrees.
    /// A reference ground position must be known:
    ///   - either it is defined in [Header]
    ///   - otherwise it can be superceeded by user defined position
    ///   - if none of these conditions are matched, method will panic
    /// ```
    /// use rinex::wgs84;
    /// use rinex::prelude::*;
    /// let ref_pos = wgs84!(3582105.291, 532589.7313, 5232754.8054);
    ///
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/ESBC00DNK_R_20201770000_01D_MN.rnx.gz")
    ///     .unwrap();
    ///
    /// let data = rinex.sv_elevation_azimuth(Some(ref_pos));
    /// for (epoch, (sv, (elev, azim))) in data {
    ///     // azim: azimuth in °
    ///     // elev: elevation in °
    /// }
    /// ```
    pub fn sv_elevation_azimuth(
        &self,
        ref_position: Option<GroundPosition>,
    ) -> Box<dyn Iterator<Item = (Epoch, (Sv, (f64, f64)))> + '_> {
        let ground_position = match ref_position {
            Some(pos) => pos, // user value superceeds, in case it is passed
            _ => {
                // header must contain this information
                // otherwise, calculation is not feasible
                if let Some(pos) = self.header.ground_position {
                    pos
                } else {
                    panic!("sv_elevation_azimuth(): needs a reference position");
                }
            },
        };
        Box::new(
            self.ephemeris()
                .filter_map(move |(epoch, (_, sv, ephemeris))| {
                    if let Some((elev, azim)) = ephemeris.sv_elev_azim(sv, *epoch, ground_position)
                    {
                        Some((*epoch, (*sv, (elev, azim))))
                    } else {
                        None // calculations may not be feasible,
                             // mainly when mandatory ephemeris broadcasts are missing
                    }
                }),
        )
    }
    /// Returns [`IonMessage`] frames Iterator
    pub fn ionosphere_models(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, &Sv, &IonMessage))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, ion)) = fr.as_ion() {
                    Some((e, (msg, sv, ion)))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns [`KbModel`] Iterator
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::navigation::KbRegionCode;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, kb_model) in rnx.klobuchar_models() {
    ///     let alpha = kb_model.alpha;
    ///     let beta = kb_model.beta;
    ///     // we only have this example at the moment
    ///     assert_eq!(kb_model.region, KbRegionCode::WideArea);
    /// }
    /// ```
    pub fn klobuchar_models(&self) -> Box<dyn Iterator<Item = (Epoch, KbModel)> + '_> {
        Box::new(
            self.ionosphere_models()
                .filter_map(|(e, (_, _, ion))| ion.as_klobuchar().map(|model| (*e, *model))),
        )
    }
    /// Returns [`NgModel`] Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, ng_model) in rnx.nequick_g_models() {
    ///     let (a0, a1, a2) = ng_model.a;
    ///     let region = ng_model.region; // bitflag: supports bitmasking operations
    /// }
    /// ```
    pub fn nequick_g_models(&self) -> Box<dyn Iterator<Item = (Epoch, NgModel)> + '_> {
        Box::new(
            self.ionosphere_models()
                .filter_map(|(e, (_, _, ion))| ion.as_nequick_g().map(|model| (*e, *model))),
        )
    }
    /// Returns [`BdModel`] Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, bd_model) in rnx.bdgim_models() {
    ///     let alpha_tecu = bd_model.alpha;
    /// }
    /// ```
    pub fn bdgim_models(&self) -> Box<dyn Iterator<Item = (Epoch, BdModel)> + '_> {
        Box::new(
            self.ionosphere_models()
                .filter_map(|(e, (_, _, ion))| ion.as_bdgim().map(|model| (*e, *model))),
        )
    }
    /// Returns [`StoMessage`] frames Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, (msg, sv, data)) in rnx.system_time_offset() {
    ///    let system = data.system.clone(); // time system
    ///    let utc = data.utc.clone(); // UTC provider
    ///    let t_tm = data.t_tm; // message transmission time in week seconds
    ///    let (a, dadt, ddadt) = data.a;
    /// }
    /// ```
    pub fn system_time_offset(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, &Sv, &StoMessage))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, sto)) = fr.as_sto() {
                    Some((e, (msg, sv, sto)))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns [`EopMessage`] frames Iterator
    /// ```
    /// use rinex::prelude::*;
    /// let rnx = Rinex::from_file("../test_resources/NAV/V4/KMS300DNK_R_20221591000_01H_MN.rnx.gz")
    ///     .unwrap();
    /// for (epoch, (msg, sv, eop)) in rnx.earth_orientation() {
    ///     let (x, dxdt, ddxdt) = eop.x;
    ///     let (y, dydt, ddydt) = eop.x;
    ///     let t_tm = eop.t_tm;
    ///     let (u, dudt, ddudt) = eop.delta_ut1;
    /// }
    /// ```
    pub fn earth_orientation(
        &self,
    ) -> Box<dyn Iterator<Item = (&Epoch, (NavMsgType, &Sv, &EopMessage))> + '_> {
        Box::new(self.navigation().flat_map(|(e, frames)| {
            frames.iter().filter_map(move |fr| {
                if let Some((msg, sv, eop)) = fr.as_eop() {
                    Some((e, (msg, sv, eop)))
                } else {
                    None
                }
            })
        }))
    }
}

/*
 * Meteo RINEX specific methods: only available on crate feature.
 * Either specific Iterators, or meaningful data we can extract.
 */
#[cfg(feature = "meteo")]
#[cfg_attr(docrs, doc(cfg(feature = "meteo")))]
impl Rinex {
    /// Returns temperature data iterator, values expressed in Celcius degrees
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, tmp) in rinex.temperature() {
    ///     println!("ts: {}, value: {} °C", epoch, tmp);
    /// }
    /// ```
    pub fn temperature(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::Temperature {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns pressure data iterator, values expressed in hPa
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, p) in rinex.pressure() {
    ///     println!("ts: {}, value: {} hPa", epoch, p);
    /// }
    /// ```
    pub fn pressure(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::Pressure {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns moisture rate iterator, values expressed in saturation rate percentage
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.moisture() {
    ///     println!("ts: {}, value: {} %", epoch, value);
    /// }
    /// ```
    pub fn moisture(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::HumidityRate {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns wind speed observations iterator, values in m/s
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, speed) in rinex.wind_speed() {
    ///     println!("ts: {}, value: {} m/s", epoch, speed);
    /// }
    /// ```
    pub fn wind_speed(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::WindSpeed {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns wind direction observations as azimuth in degrees
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, azimuth) in rinex.wind_direction() {
    ///     println!("ts: {}, azimuth: {}°", epoch, azimuth);
    /// }
    /// ```
    pub fn wind_direction(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::WindDirection {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns rain increment observations iterator, values in tenth of mm.
    /// Each value represents the accumulated rain drop in between two observations.
    /// ```
    /// use rinex::prelude::*;
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, ri) in rinex.rain_increment() {
    ///     println!("ts: {}, accumulated: {} mm/10", epoch, ri);
    /// }
    /// ```
    pub fn rain_increment(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::RainIncrement {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns total (wet+dry) Zenith delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithTotalDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns Zenith dry delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_dry_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_dry_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithDryDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns Zenith wet delay, in mm
    /// ```
    /// use rinex::prelude::*;
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// for (epoch, value) in rinex.zenith_wet_delay() {
    ///     println!("ts: {}, value: {} mm", epoch, value);
    /// }
    /// ```
    pub fn zenith_wet_delay(&self) -> Box<dyn Iterator<Item = (Epoch, f64)> + '_> {
        Box::new(self.meteo().flat_map(|(epoch, v)| {
            v.iter().filter_map(|(k, value)| {
                if *k == Observable::ZenithWetDelay {
                    Some((*epoch, *value))
                } else {
                    None
                }
            })
        }))
    }
    /// Returns true if rain was detected during this time frame.
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T20:00:00 UTC"));
    /// assert_eq!(rinex.rain_detected(), false);
    /// ```
    pub fn rain_detected(&self) -> bool {
        for (_, ri) in self.rain_increment() {
            if ri > 0.0 {
                return true;
            }
        }
        false
    }
    /// Returns total accumulated rain in tenth of mm, within this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T19:30:00 UTC"));
    /// assert_eq!(rinex.accumulated_rain(), 0.0);
    /// assert_eq!(rinex.rain_detected(), false);
    /// ```
    pub fn accumulated_rain(&self) -> f64 {
        self.rain_increment()
            .zip(self.rain_increment().skip(1))
            .fold(0_f64, |mut acc, ((_, rk), (_, rkp1))| {
                if acc == 0.0_f64 {
                    acc = rkp1; // we take r(0) as starting offset
                } else {
                    acc += rkp1 - rk; // then accumulate the deltas
                }
                acc
            })
    }
    /// Returns true if hail was detected during this time frame
    /// ```
    /// use std::str::FromStr;
    /// use rinex::{filter, Rinex};
    /// use rinex::preprocessing::*; // .filter()
    /// let mut rinex = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
    ///     .unwrap();
    /// // cropping time frame requires the "processing" feature
    /// let rinex = rinex
    ///                 .filter(filter!(">= 2015-01-01T19:00:00 UTC"))
    ///                 .filter(filter!(" < 2015-01-01T20:00:00 UTC"));
    /// assert_eq!(rinex.hail_detected(), false);
    /// ```
    pub fn hail_detected(&self) -> bool {
        if let Some(r) = self.record.as_meteo() {
            for (_, observables) in r {
                for (observ, value) in observables {
                    if *observ == Observable::HailIndicator {
                        if *value > 0.0 {
                            return true;
                        }
                    }
                }
            }
            false
        } else {
            false
        }
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
        Ok(())
        //TODO: needs to reapply
        //if self.epoch().len() == 0 {
        //    // self is empty
        //    self.record = rhs.record.clone();
        //    Ok(())
        //} else if rhs.epoch().len() == 0 {
        //    // nothing to merge
        //    Ok(())
        //} else {
        //    // add special marker, ts: YYYYDDMM HHMMSS UTC
        //    let now = hifitime::Epoch::now().expect("failed to retrieve system time");
        //    let (y, m, d, hh, mm, ss, _) = now.to_gregorian_utc();
        //    self.header.comments.push(format!(
        //        "rustrnx-{:<20} FILE MERGE          {}{}{} {}{}{} {}",
        //        env!("CARGO_PKG_VERSION"),
        //        y + 1900,
        //        m,
        //        d,
        //        hh,
        //        mm,
        //        ss,
        //        now.time_scale
        //    ));
        //    // RINEX record merging
        //    self.record.merge_mut(&rhs.record)?;
        //    Ok(())
        //}
    }
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

#[cfg(feature = "processing")]
use preprocessing::Smooth;

#[cfg(feature = "processing")]
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

#[cfg(feature = "processing")]
use crate::algorithm::{Filter, Preprocessing};

#[cfg(feature = "processing")]
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

#[cfg(feature = "processing")]
use crate::algorithm::Decimate;

#[cfg(feature = "processing")]
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

//#[cfg(feature = "obs")]
//use observation::Mp;
//
//#[cfg(feature = "obs")]
//impl Mp for Rinex {
//    fn mp(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>> {
//        /*
//         * Determine mean value of all observed Phase and Pseudo Range
//         * observations, for all Sv
//         */
//        let mut mean: HashMap<Sv, HashMap<Observable, f64>> = HashMap::new();
//        /*
//         * Associate a Phase code to all PR codes
//         */
//        let mut associations: HashMap<Observable, Observable> = HashMap::new();
//
//        let pr_codes: Vec<Observable> = self.observable()
//            .filter_map(|obs|
//                if obs.is_pseudorange_observable() {
//                    Some(obs.clone())
//                } else {
//                    None
//                })
//            .collect();
//
//        for observable in self.observable() {
//            if !observable.is_phase_observable() {
//                if !observable.is_pseudorange_observable() {
//                    continue;
//                }
//            }
//            // code associations (for future combianations)
//            if observable.is_phase_observable() {
//                for pr_code in &pr_codes {
//
//                }
//            }
//
//            for sv in self.sv() {
//                /*
//                 * average phase values
//                 */
//                let phases: Vec<f64> = self.carrier_phase()
//                    .filter_map(|(_, svnn, obs, ph)| {
//                        if sv == svnn && observable == obs {
//                            Some(ph)
//                        } else {
//                            None
//                        }
//                    })
//                    .collect();
//                if let Some(data) = mean.get_mut(&sv) {
//                    data.insert(observable.clone(), phases.mean());
//                } else {
//                    let mut map: HashMap<Observable, f64> = HashMap::new();
//                    map.insert(observable.clone(), phases.mean());
//                    mean.insert(sv, map);
//                }
//                /*
//                 * average PR values
//                 */
//                let pr: Vec<f64> = self.pseudo_range()
//                    .filter_map(|(_, svnn, obs, pr)| {
//                        if sv == svnn && observable == obs {
//                            Some(pr)
//                        } else {
//                            None
//                        }
//                    })
//                    .collect();
//                if let Some(data) = mean.get_mut(&sv) {
//                    data.insert(observable.clone(), pr.mean());
//                } else {
//                    let mut map: HashMap<Observable, f64> = HashMap::new();
//                    map.insert(observable.clone(), pr.mean());
//                    mean.insert(sv, map);
//                }
//            }
//        }
//        HashMap::new()
//    }
//}

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

#[cfg(feature = "obs")]
use observation::IonoDelay;

#[cfg(feature = "obs")]
impl IonoDelay for Rinex {
    fn iono_delay(
        &self,
        max_dt: Duration,
    ) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>> {
        if let Some(r) = self.record.as_obs() {
            r.iono_delay(max_dt)
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
