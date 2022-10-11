//! This library provides a set of tools to parse, analyze,
//! produce and manipulate `RINEX` files.  
//! Refer to README and official documentation, extensive examples of use
//! are provided.  
//! Homepage: <https://github.com/gwbres/rinex>
mod leap;
mod merge;
mod formatter;
//mod gnss_time;

pub mod antex;
pub mod channel;
pub mod clocks;
pub mod constellation;
pub mod hardware;
pub mod hatanaka;
pub mod header;
pub mod ionosphere;
pub mod meteo;
pub mod navigation;
pub mod observation;
pub mod record;
pub mod sv;
pub mod epoch;
pub mod types;
pub mod version;
//pub mod processing;

extern crate num;
#[macro_use]
extern crate num_derive;

pub mod reader;
use reader::BufferedReader;
use std::io::{Write}; //, Read};

pub mod writer;
use writer::BufferedWriter;

use thiserror::Error;
use chrono::{Datelike, Timelike};
use std::collections::{BTreeMap, HashMap};

use navigation::OrbitItem;

// export major types
pub use sv::Sv;
pub use header::Header;
pub use epoch::{Epoch, EpochFlag};
pub use constellation::Constellation;
//pub use processing::{DiffContext, Context1D};

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_comment {
    ($line: expr) => { $line.trim_end().ends_with("COMMENT") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a pseudo range (OBS) code
macro_rules! is_pseudo_range_obs_code {
    ($code: expr) => { 
        $code.starts_with("C") // standard 
        || $code.starts_with("P") // non gps old fashion
    };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a phase (OBS) code
macro_rules! is_phase_carrier_obs_code {
    ($code: expr) => { $code.starts_with("L") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a doppler (OBS) code
macro_rules! is_doppler_obs_code {
    ($code: expr) => { $code.starts_with("D") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a signal strength (OBS) code
macro_rules! is_sig_strength_obs_code {
    ($code: expr) => { $code.starts_with("S") };
}

/// Returns `str` description, as one letter
/// lowercase, used in RINEX file name to describe 
/// the sampling period. RINEX specifications:   
/// “a” = 00:00:00 - 00:59:59   
/// “b” = 01:00:00 - 01:59:59   
/// [...]   
/// "x" = 23:00:00 - 23:59:59
/// This method expects a chrono::NaiveDateTime as an input
fn hourly_session_str (time: chrono::NaiveTime) -> String {
    let h = time.hour() as u8;
    if h == 23 {
        String::from("x")
    } else {
        let c : char = (h+97).into();
        String::from(c)
    }
}

/// `Rinex` describes a `RINEX` file
#[derive(Clone, Debug)]
#[derive(PartialEq)]
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

#[derive(Error, Debug)]
/// `Split` ops related errors
pub enum SplitError {
    #[error("desired epoch is too early")]
    EpochTooEarly,
    #[error("desired epoch is too late")]
    EpochTooLate,
}

#[derive(Error, Debug)]
/// `Diff` (RNX(A) - RNX(B)) related errors
pub enum DiffError {
    #[error("not an observation rinex")]
    NotObsRinex,
}

impl std::ops::Sub<Rinex> for Rinex {
    type Output = Self;
    fn sub (self, rhs: Self) -> Self {
        self.observation_diff(&rhs).unwrap()
    }
}

impl std::ops::SubAssign<Rinex> for Rinex {
    fn sub_assign (&mut self, rhs: Self) {
        self.observation_diff_mut(&rhs).unwrap()
    }
}

impl Rinex {
    /// Builds a new `RINEX` struct from given header & body sections
    pub fn new (header: Header, record: record::Record) -> Rinex {
        Rinex {
            header,
            record,
            comments: record::Comments::new(),
        }
    }

    /// Returns a copy of self with given header attributes
    pub fn with_header (&self, header: Header) -> Self {
        Rinex {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
        }
    }

    /// Replaces header section
    pub fn replace_header (&mut self, header: Header) {
        self.header = header.clone();
    }

    /// Returns a copy of self with given internal record
    pub fn with_record (&self, record: record::Record) -> Self {
        Rinex {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record: record,
        }
    }

    /// Replaces internal record
    pub fn replace_record (&mut self, record: record::Record) {
        self.record = record.clone();
    }

    /// Converts self to CRINEX compatible format.
    /// This is useful in case we parsed some compressed
    /// data that we want to uncompress.
    /// This has no effect if self is not an Observation RINEX,
    /// because it is not clear to this day, if CRINEX compression
    /// is feasible on other types of RINEX.
    ///
    /// Example
    /// ```
    /// use rinex::*;
    /// // grab a CRINEX (compressed RINEX file)
    /// let mut rnx = Rinex::from_file("../test_resources/CRNX/V3/KUNZ00CZE.crx").unwrap();
    /// // let's assume you are interested in file production
    /// // here we customize rnx` a little bit
    /// rnx
    ///     .header
    ///         .with_general_infos("my_prog", "runby", "my_agency");
    /// rnx.to_file("/tmp/KUNZ00CSZ.crx"); // this will produce the same format, 
    ///                               // --> data is compressed
    /// rnx.crx2rnx(); // by doing this we move to uncompressed data production all CRINEX attributes 
    /// rnx.to_file("/tmp/KUNZ00CSZ.rnx"); // --> data is readable 
    /// ```
    pub fn crx2rnx (&mut self) {
        if self.is_observation_rinex() {
            let now = chrono::Utc::now().naive_utc();
            self.header = self.header
                .with_crinex(
                    observation::Crinex {
                        version: version::Version {
                            major: 3, // latest CRINEX
                            minor: 0, // latest CRINEX
                        },
                        prog: "rustcrx".to_string(),
                        date: now.date().and_time(now.time()),
                    })
        }
    }

    /// Returns filename that would respect naming conventions,
    /// based on self attributes
    pub fn filename (&self) -> String {
        let header = &self.header;
        let rtype = header.rinex_type;
        let nnnn = header.station.as_str()[0..4].to_lowercase(); 
        //TODO:
        //self.header.date should be a datetime object
        //but it is complex to parse..
        let ddd = String::from("DDD"); 
        let epoch : Epoch = match rtype {
              types::Type::ObservationData 
            | types::Type::NavigationData 
            | types::Type::MeteoData 
            | types::Type::ClockData => self.epochs()[0],
            _ => todo!(), // other files require a dedicated procedure
        };
        if header.version.major < 3 {
            let s = hourly_session_str(epoch.date.time());
            let yy = format!("{:02}", epoch.date.year());
            let t : String = match rtype {
                types::Type::ObservationData => {
                    if header.is_crinex() {
                        String::from("d")
                    } else {
                        String::from("o")
                    }
                },
                types::Type::NavigationData => {
                    if let Some(c) = header.constellation {
                        if c == constellation::Constellation::Glonass {
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
            let yyyy = format!("{:04}", epoch.date.year());
            let hh = format!("{:02}", epoch.date.hour());
            let mm = format!("{:02}", epoch.date.minute());
            let pp = String::from("00"); //TODO 02d file period, interval ?
            let up = String::from("H"); //TODO: file period unit
            let ff = String::from("00"); //TODO: 02d observation frequency 02d
            //TODO
            //Units of frequency FF. “C” = 100Hz; “Z” = Hz; “S” = sec; “M” = min;
            //“H” = hour; “D” = day; “U” = unspecified
            //NB - _FFU is omitted for files containing navigation data
            let uf = String::from("Z");
            let c : String = match header.constellation {
                Some(c) => c.to_1_letter_code().to_uppercase(),
                _ => String::from("X"),
            };
            let t : String = match rtype {
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
            format!("{}{}{}{}_{}_{}{}{}{}_{}{}_{}{}_{}{}.{}",
                nnnn, m, r, ccc, s, yyyy, ddd, hh, mm, pp, up, ff, uf, c, t, fmt)
        }
    }

    /// Builds a `RINEX` from given file.
    /// Header section must respect labelization standards, 
    /// some are mandatory.   
    /// Parses record (file body) for supported `RINEX` types.
    pub fn from_file (path: &str) -> Result<Rinex, Error> {
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
        let header = Header::new(&mut reader)
            .unwrap();
        // --> parse record (file body)
        //     we also grab encountered comments,
        //     they might serve some fileops like `splice` / `merge` 
        let (record, comments) = record::parse_record(&mut reader, &header)
            .unwrap();
        Ok(Rinex {
            header,
            record,
            comments,
        })
    }

    /// Returns true if this is an ATX RINEX 
    pub fn is_antex_rinex (&self) -> bool { self.header.rinex_type == types::Type::AntennaData }
    
    /// Returns true if this is a CLOCK RINEX
    pub fn is_clocks_rinex (&self) -> bool { self.header.rinex_type == types::Type::ClockData }

    /// Returns true if this is an IONEX file
    pub fn is_ionex (&self) -> bool { self.header.rinex_type == types::Type::IonosphereMaps }

    /// Returns true if this is a METEO RINEX
    pub fn is_meteo_rinex (&self) -> bool { self.header.rinex_type == types::Type::MeteoData }
    
    /// Retruns true if this is a NAV RINEX
    pub fn is_navigation_rinex (&self) -> bool { self.header.rinex_type == types::Type::NavigationData }

    /// Retruns true if this is an OBS RINEX
    pub fn is_observation_rinex (&self) -> bool { self.header.rinex_type == types::Type::ObservationData }

    /// Returns `epoch` of first observation
    pub fn first_epoch (&self) -> Option<Epoch> {
        let epochs = self.epochs();
        if epochs.len() == 0 {
            None
        } else {
            Some(epochs[0])
        }
    }

    /// Returns `epoch` of last observation
    pub fn last_epoch (&self) -> Option<Epoch> {
        let epochs = self.epochs();
        if epochs.len() == 0 {
            None
        } else {
            Some(epochs[epochs.len()-1])
        }
    }

    /// Returns sampling interval of this record
    /// either directly from header section, if such information was provided,
    /// or the most encountered epoch interval.
    ///
    /// Example:
    /// ```
    /// use rinex::*;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
    /// // in this file, header section contains desired information directly
    /// assert_eq!(rnx.sampling_interval().num_seconds(), rnx.header.sampling_interval.unwrap() as i64);
    /// let rnx = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx").unwrap();
    /// // in that file, we had to compute that information ourselves
    /// assert_eq!(rnx.header.sampling_interval, None);
    /// //01 00 00 00
    /// //01 00 15 00 --> 15'
    /// //01 05 00 00 --> 4h45
    /// //01 09 45 00 --> 4h45
    /// //01 10 00 00 --> 15'
    /// //01 15 40 00 --> 5h40
    /// //--------------> 15' is the most "plausible"
    /// let expected = chrono::Duration::from_std(std::time::Duration::from_secs(15*60)).unwrap();
    /// //assert_eq!(rnx.sampling_interval(), expected);
    /// ```
    pub fn sampling_interval (&self) -> chrono::Duration {
        if let Some(interval) = self.header.sampling_interval {
            let secs = interval.round() as u64;
            //let nanos = interval.fract() as i32;
            chrono::Duration::from_std(std::time::Duration::from_secs(secs)).unwrap()
        } else {
            let epochs = self.epochs();
            let mut prev_epoch = epochs[0];
            let mut histogram : HashMap<chrono::Duration, u64> = HashMap::new();
            for epoch in epochs.iter().skip(1) {
                let dt = epoch.date - prev_epoch.date;
                if let Some(counter) = histogram.get_mut(&dt) {
                    *counter += 1
                } else {
                    histogram.insert(dt, 1);
                }
                prev_epoch = epoch.clone();
            }
            // largest hist population
            let mut largest = 0;
            let mut dt = chrono::Duration::from_std(std::time::Duration::from_secs(0)).unwrap();
            for (d, counter) in histogram.iter() {
                if counter > &largest {
                    largest = *counter;
                    dt = d.clone()
                } else if counter == &largest {
                    if d < &dt { // on population equality --> smallest epoch interval is preferred
                        dt = d.clone()
                    }
                }
            }
            dt
        }
    }

    /// Returns start date and duration of largest data gap in this record.
    /// Returns None if no data gap was found (ie., all epochs comprised within expected sampling interval)
    pub fn largest_data_gap_duration (&self) -> Option<(chrono::NaiveDateTime, chrono::Duration)> {
        if let Some(interval) = self.header.sampling_interval {
            let epochs = self.epochs();
            let interval = chrono::Duration::from_std(std::time::Duration::from_secs(interval as u64)).unwrap();
            let mut prev_epoch = epochs[0];
            let mut epoch = epochs[0].date.clone();
            let mut largest_dt = prev_epoch.date - prev_epoch.date;
            for e in epochs.iter().skip(1) {
                let dt = e.date - prev_epoch.date;
                if dt > interval {
                    if dt > largest_dt { 
                        epoch = e.date.clone();
                        largest_dt = e.date - prev_epoch.date; 
                    }
                }
                prev_epoch = e.clone(); 
            }
            Some((epoch, largest_dt))
        } else {
            None
        }
    }

    /// Returns a list of epochs that present a data gap.
    /// Data gap is determined by comparing |e(k)-e(k-1)|: successive epoch intervals,
    /// to the INTERVAL field found in the header.
    /// Granularity is currently limited to 1 second. 
    /// This method will not produce anything if header does not an INTERVAL field.
    pub fn data_gaps (&self) -> Vec<Epoch> {
        if let Some(interval) = self.header.sampling_interval {
            let interval = interval as u64;
            let mut epochs = self.epochs();
            let mut prev = epochs[0].date;
            epochs
                .retain(|e| {
                    let delta = (e.date - prev).num_seconds() as u64; 
                    if delta <= interval {
                        prev = e.date;
                        true
                    } else {
                        false
                    }
            });
            epochs
        } else {
            Vec::new()
        }
    }
    
    /// Returns list of epochs where unusual events happened,
    /// ie., epochs with an != Ok flag attached to them. 
    /// This method does not filter anything on non Observation Records. 
    /// This method is very useful to determine when special/external events happened
    /// and what kind of events happened, such as:  
    ///  -  power cycle failures
    ///  - receiver physically moved (new site occupation)
    ///  - other external events 
    pub fn epoch_anomalies (&self, mask: Option<EpochFlag>) -> Vec<Epoch> { 
        let epochs = self.epochs();
        epochs
            .into_iter()
            .filter(|e| {
                let mut nok = !e.flag.is_ok(); // abnormal epoch
                if let Some(mask) = mask {
                    nok &= e.flag == mask // + match specific event mask
                }
                nok
            })
            .collect()
    }

    /// Returns (if possible) event explanation / description by searching through identified comments,
    /// and returning closest comment (inside record) in time.    
    /// Usually, comments are associated to epoch events (anomalies) to describe what happened.   
    /// This method tries to locate a list of comments that were associated to the given timestamp 
    pub fn event_description (&self, event: Epoch) -> Option<&str> {
        let comments : Vec<_> = self.comments
            .iter()
            .filter(|(k,_)| *k == &event)
            .map(|(_,v)| v)
            .flatten()
            .collect();
        if comments.len() > 0 {
            Some(comments[0]) // TODO grab all content! by serializing into a single string
        } else {
            None
        }
    } 

    /// Returns `true` if self is a `merged` RINEX file,   
    /// meaning, this file is the combination of two RINEX files merged together.  
    /// This is determined by the presence of a custom yet somewhat standardized `FILE MERGE` comments
    pub fn is_merged (&self) -> bool {
        for (_, content) in self.comments.iter() {
            for c in content {
                if c.contains("FILE MERGE") {
                    return true
                }
            }
        }
        false
    }

    /// Returns list of epochs where RINEX merging operation(s) occurred.    
    /// Epochs are determined either by the pseudo standard `FILE MERGE` comment description.
    pub fn merge_boundaries (&self) -> Vec<chrono::NaiveDateTime> {
        self.header
            .comments
            .iter()
            .flat_map(|s| {
                if s.contains("FILE MERGE") {
                    let content = s.split_at(40).1.trim();
                    if let Ok(date) = chrono::NaiveDateTime::parse_from_str(content, "%Y%m%d %h%m%s UTC") {
                        Some(date)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Splits self into several RINEXes if self is a Merged Rinex. 
    /// Header sections are simply copied.
    pub fn split (&self) -> Vec<Self> {
        let records = self.split_merged_records();
        let mut result :Vec<Self> = Vec::with_capacity(records.len());
        for r in records {
            result.push(Self {
                header: self.header.clone(),
                record: r.clone(),
                comments: self.comments.clone(),
            })
        }
        result
    }
    
    /// Splits `merged` records into seperate records.
    /// Returns empty list if self is not a `Merged` file
    pub fn split_merged_records (&self) -> Vec<record::Record> {
        let boundaries = self.merge_boundaries();
        let mut result : Vec<record::Record> = Vec::with_capacity(boundaries.len());
        let epochs = self.epochs();
        let mut e0 = epochs[0].date;
        for boundary in boundaries {
            let rec : record::Record = match self.header.rinex_type {
                types::Type::NavigationData => {
                    let mut record = self.record
                        .as_nav()
                        .unwrap()
                        .clone();
                    record.retain(|e, _| e.date >= e0 && e.date < boundary);
                    record::Record::NavRecord(record.clone())
                },
                types::Type::ObservationData => {
                    let mut record = self.record
                        .as_obs()
                        .unwrap()
                        .clone();
                    record.retain(|e, _| e.date >= e0 && e.date < boundary);
                    record::Record::ObsRecord(record.clone())
                },
                types::Type::MeteoData => {
                    let mut record = self.record
                        .as_meteo()
                        .unwrap()
                        .clone();
                    record.retain(|e, _| e.date >= e0 && e.date < boundary);
                    record::Record::MeteoRecord(record.clone())
                },
                types::Type::IonosphereMaps => {
                    let mut record = self.record
                        .as_ionex()
                        .unwrap()
                        .clone();
                    record.retain(|e, _| e.date >= e0 && e.date < boundary);
                    record::Record::IonexRecord(record.clone())
                },
                _ => todo!("implement other record types"),
            };
            result.push(rec);
            e0 = boundary 
        }
        result
    }

    /// Splits self into two RINEXes, at desired epoch.
    /// Header sections are simply copied.
    pub fn split_at_epoch (&self, epoch: Epoch) -> Result<(Self, Self), SplitError> {
        let (r0, r1) = self.split_record_at_epoch(epoch)?;
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


    /// Splits record into two at desired `epoch`.
    /// Self does not have to be a `Merged` file.
    pub fn split_record_at_epoch (&self, epoch: Epoch) -> Result<(record::Record,record::Record), SplitError> {
        let epochs = self.epochs();
        if epoch.date < epochs[0].date {
            return Err(SplitError::EpochTooEarly)
        }
        if epoch.date > epochs[epochs.len()-1].date {
            return Err(SplitError::EpochTooLate)
        }
        let rec0 : record::Record = match self.header.rinex_type {
            types::Type::NavigationData => {
                let rec = self.record.as_nav()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date < epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::NavRecord(rec)
            },
            types::Type::ObservationData => {
                let rec = self.record.as_obs()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date < epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::ObsRecord(rec)
            },
            types::Type::MeteoData => {
                let rec = self.record.as_meteo()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date < epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::MeteoRecord(rec)
            },
            _ => unreachable!("epochs::iter()"),
        };
        let rec1 : record::Record = match self.header.rinex_type {
            types::Type::NavigationData => {
                let rec = self.record.as_nav()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date >= epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::NavRecord(rec)
            },
            types::Type::ObservationData => {
                let rec = self.record.as_obs()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date >= epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::ObsRecord(rec)
            },
            types::Type::MeteoData => {
                let rec = self.record.as_meteo()
                    .unwrap()
                        .iter()
                        .flat_map(|(k, v)| {
                            if k.date >= epoch.date {
                                Some((k, v))
                            } else {
                                None
                            }
                        })
                        .map(|(k,v)| (k.clone(),v.clone())) // BTmap collect() derefencing 
                        .collect();
                record::Record::MeteoRecord(rec)
            },
            _ => unreachable!("epochs::iter()"),
        };
        Ok((rec0,rec1))
    }

    /// Returns list of epochs contained in self.
    /// Faillible! if this RINEX is not indexed by `epochs`
    pub fn epochs (&self) -> Vec<Epoch> {
        match self.header.rinex_type {
            types::Type::ObservationData => {
                self.record
                    .as_obs()
                    .unwrap()
                    .into_iter()
                    .map(|(k, _)| *k)
                    .collect()
            },
            types::Type::NavigationData => {
                self.record
                    .as_nav()
                    .unwrap()
                    .into_iter()
                    .map(|(k, _)| *k)
                    .collect()
            },
            types::Type::MeteoData => {
                self.record
                    .as_meteo()
                    .unwrap()
                    .into_iter()
                    .map(|(k, _)| *k)
                    .collect()
            },
            types::Type::ClockData => {
                self.record
                    .as_clock()
                    .unwrap()
                    .into_iter()
                    .map(|(k, _)| *k)
                    .collect()
            },
            types::Type::IonosphereMaps => {
                self.record
                    .as_ionex()
                    .unwrap()
                    .into_iter()
                    .map(|(k, _)| *k)
                    .collect()
            },
            _ => panic!("Cannot get an epoch iterator for \"{:?}\"", self.header.rinex_type),
        }
    }

    /// Merges given RINEX into self, in teqc similar fashion.   
    /// Header sections are combined (refer to header::merge Doc
    /// to understand its behavior).
    /// Resulting self.record (modified in place) remains sorted by 
    /// sampling timestamps.
    pub fn merge_mut (&mut self, other: &Self) -> Result<(), merge::MergeError> {
        self.header.merge_mut(&other.header)?;
        // grab Self:: + Other:: `epochs`
        let (epochs, other_epochs) = (self.epochs(), other.epochs());
        if epochs.len() == 0 { // self is empty
            self.record = other.record.clone();
            Ok(()) // --> self is overwritten
        } else if other_epochs.len() == 0 { // nothing to merge
            Ok(()) // --> self is untouched
        } else {
            // add Merge op descriptor
            let now = chrono::offset::Utc::now();
            self.header.comments.push(format!(
                "rustrnx-{:<20} FILE MERGE          {} UTC", 
                env!("CARGO_PKG_VERSION"),
                now.format("%Y%m%d %H%M%S")));
            // merge op
            match self.header.rinex_type {
                types::Type::NavigationData => {
                    let a_rec = self.record
                        .as_mut_nav()
                        .unwrap();
                    let b_rec = other.record
                        .as_nav()
                        .unwrap();
                    for (k, v) in b_rec {
                        a_rec.insert(*k, v.clone());
                    }
                },
                types::Type::ObservationData => {
                    let a_rec = self.record
                        .as_mut_obs()
                        .unwrap();
                    let b_rec = other.record
                        .as_obs()
                        .unwrap();
                    for (k, v) in b_rec {
                        a_rec.insert(*k, v.clone());
                    }
                },
                types::Type::MeteoData => {
                    let a_rec = self.record
                        .as_mut_meteo()
                        .unwrap();
                    let b_rec = other.record
                        .as_meteo()
                        .unwrap();
                    for (k, v) in b_rec {
                        a_rec.insert(*k, v.clone());
                    }
                },
                types::Type::IonosphereMaps => {
                    let a_rec = self.record
                        .as_mut_ionex()
                        .unwrap();
                    let b_rec = other.record
                        .as_ionex()
                        .unwrap();
                    for (k, v) in b_rec {
                        a_rec.insert(*k, v.clone());
                    }
                },
                _ => unreachable!("epochs::iter()"),
            }
            Ok(())
        }
    }
    
    /// [merge] immutable implementation
    pub fn merge (&self, rhs: &Self) -> Result<Self, merge::MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    
    /// Retains only data that have an Ok flag associated to them. 
    pub fn retain_epoch_ok_mut (&mut self) {
        if !self.is_observation_rinex() {
            return ; // nothing to browse
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record.retain(|e, _| e.flag.is_ok());
    }

    /// Filters out epochs that do not have an Ok flag associated
    /// to them. This can be due to events like Power Failure,
    /// Receiver or antenna being moved.. See [epoch::EpochFlag].
    pub fn retain_epoch_nok_mut (&mut self) {
        if !self.is_observation_rinex() {
            return ; // nothing to browse
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record.retain(|e, _| !e.flag.is_ok())
    }
    
    /// Immutable implementation of [retain_epoch_ok_mut]
    pub fn retain_epoch_ok (&self) -> Self {
        let mut s = self.clone();
        s.retain_epoch_ok_mut();
        s
    }
    
    /// Immutable implementation [retain_epoch_nok_mut]
    pub fn retain_epoch_nok (&self) -> Self {
        let mut s = self.clone();
        s.retain_epoch_nok_mut();
        s
    }
    
    /// Returns epochs where a loss of lock event happened.
    /// Has no effects on non Observation Records.
    pub fn observation_epoch_lock_loss (&self) -> Vec<Epoch> {
        self
            .observation_lli_and_mask(observation::LliFlags::LOCK_LOSS)
            .epochs()
    }

    /// Removes all observations where lock condition was lost
    pub fn observation_lock_loss_filter_mut (&mut self) {
        self
            .observation_lli_and_mask_mut(observation::LliFlags::LOCK_LOSS)
    }

    /// List all constellations contained in record
    pub fn list_constellations (&self) -> Vec<constellation::Constellation> {
        let mut ret: Vec<constellation::Constellation> = Vec::new();
        match self.header.constellation {
            Some(constellation::Constellation::Mixed) => {
                match self.header.rinex_type {
                    types::Type::ObservationData => {
                        let record = self.record
                            .as_obs()
                            .unwrap();
                        for (_e, (_clk, vehicules)) in record.iter() {
                            for (sv, _) in vehicules.iter() {
                                if !ret.contains(&sv.constellation) {
                                    ret.push(sv.constellation.clone());
                                }
                            }
                        }
                    },
                    types::Type::NavigationData => {
                        let record = self.record
                            .as_nav()
                            .unwrap();
                        for (_, classes) in record.iter() {
                            for (class, frames) in classes.iter() {
                                if *class == navigation::FrameClass::Ephemeris {
                                    for frame in frames.iter() {
                                        let (_, sv, _) = frame.as_eph().unwrap();
                                        if !ret.contains(&sv.constellation) {
                                            ret.push(sv.constellation.clone());
                                        }
                                    }
                                }
                            }
                        }
                    },
                    types::Type::ClockData => {
                        let record = self.record
                            .as_clock()
                            .unwrap();
                        for (_, types) in record.iter() {
                            for (_, systems) in types.iter() {
                                for (system, _) in systems.iter() {
                                    if let Some(sv) = system.as_sv() {
                                        if !ret.contains(&sv.constellation) {
                                            ret.push(sv.constellation.clone());
                                        }
                                    }
                                }
                            }
                        }
                    },
                    _ => {},
                }
            },
            Some(c) => ret.push(c),
            None => {},
        }
        ret
    }

    /// Retains data that was recorded along given constellation(s).
    /// This has no effect on ATX, MET and IONEX records, NAV 
    /// record frames other than Ephemeris, Clock frames not measured
    /// against space vehicule.
    pub fn retain_constellation_mut (&mut self, filter: Vec<constellation::Constellation>) {
        if self.is_observation_rinex() {
            let record = self.record
                .as_mut_obs()
                .unwrap();
            record
                .retain(|_, (_, vehicules)| {
                    vehicules.retain(|sv, _| filter.contains(&sv.constellation));
                    vehicules.len() > 0
                })
        } else if self.is_navigation_rinex() {
            let record = self.record
                .as_mut_nav()
                .unwrap();
            record
                .retain(|_, classes| {
                    classes.retain(|class, frames| {
                        if *class == navigation::FrameClass::Ephemeris {
                            frames.retain(|fr| {
                                let (_, sv, _) = fr.as_eph().unwrap();
                                filter.contains(&sv.constellation)
                            });
                            frames.len() > 0
                        } else {
                            true // retains non EPH 
                        }
                    });
                    classes.len() > 0
                })
        } else if self.is_clocks_rinex() {
            let record = self.record
                .as_mut_clock()
                .unwrap();
            record.retain(|_, dtypes| {
                dtypes.retain(|_, systems| {
                    systems.retain(|system, _| {
                        if let Some(sv) = system.as_sv() {
                            filter.contains(&sv.constellation)
                        } else {
                            true // retain other system types
                        }
                    });
                    systems.len() > 0
                });
                dtypes.len() > 0
            })
        }
    }

    /// Retains data that was generated / recorded against given list of 
    /// space vehicules. This has no effect on ATX, MET, IONEX records,
    /// and NAV record frames other than Ephemeris. On CLK records,
    /// we filter out data not measured against space vehicules and different vehicules.
    pub fn retain_space_vehicule_mut (&mut self, filter: Vec<Sv>) {
        if self.is_observation_rinex() {
            let record = self.record
                .as_mut_obs()
                .unwrap();
            record.retain(|_, (_, vehicules)| { 
                vehicules.retain(|sv, _| filter.contains(sv));
                vehicules.len() > 0
            })
        } else if self.is_navigation_rinex() {
            let record = self.record
                .as_mut_nav()
                .unwrap();
            record
                .retain(|_, classes| {
                    classes.retain(|class, frames| {
                        if *class == navigation::FrameClass::Ephemeris {
                            frames.retain(|fr| {
                                let (_, sv, _) = fr.as_eph().unwrap();
                                filter.contains(&sv)
                            });
                            frames.len() > 0
                        } else {
                            true // keeps non ephemeris as is
                        }
                    });
                    classes.len() > 0
                })
        } else if self.is_clocks_rinex() {
            let record = self.record
                .as_mut_clock()
                .unwrap();
            record
                .retain(|_, data_types| {
                    data_types.retain(|_, systems| {
                        systems.retain(|system, _| {
                            if let Some(sv) = system.as_sv() {
                                filter.contains(&sv)
                            } else {
                                false
                            }
                        });
                        systems.len() > 0
                    });
                    data_types.len() > 0
                })
        }
    }

    /// Immutable implementation of [retain_space_vehicule_mut]
    pub fn retain_space_vehicule (&self, filter: Vec<Sv>) -> Self {
        let mut s = self.clone();
        s.retain_space_vehicule_mut(filter);
        s
    }
    
    /// Returns list of vehicules per constellation and on an epoch basis
    /// that are closest to Zenith. This is basically a max() operation
    /// on the elevation angle, per epoch and constellation.
    /// This can only be computed on Navigation ephemeris. 
    pub fn space_vehicules_closest_to_zenith (&self) 
            -> BTreeMap<Epoch, Vec<Sv>> 
    {
        if !self.is_navigation_rinex() {
            return BTreeMap::new();
        }
        let record = self.record
            .as_nav()
            .unwrap();
        let mut ret : BTreeMap<Epoch, Vec<Sv>>
            = BTreeMap::new();
        for (e, classes) in record.iter() {
            let mut work: BTreeMap<constellation::Constellation, (f64, Sv)> = BTreeMap::new();
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::Ephemeris {
                    for frame in frames.iter() {
                        let (_, sv, ephemeris) = frame.as_eph()
                            .unwrap();
						let orbits = &ephemeris.orbits;
                        if let Some(elev) = orbits.get("e") {
                            // got an elevation angle
                            let elev = elev
                                .as_f64()
                                .unwrap();
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
            let mut inner : Vec<Sv> = Vec::new();
            for (_, (_, sv)) in work.iter() {
                inner.push(*sv);
            }
            ret.insert(*e, inner.clone());
        }
        ret 
    }

    /// Returns receiver clock offset, for all epoch such information
    /// was provided by this Observation record.
    /// Does not produce anything if self is not an OBS record.
    pub fn receiver_clock_offsets (&self) -> BTreeMap<Epoch, f64> {
        let mut map : BTreeMap<Epoch, f64> = BTreeMap::new();
        if !self.is_observation_rinex() {
            return map ;
        }
        let record = self.record
            .as_obs()
            .unwrap();
        for (epoch, (clk, _)) in record.iter() {
            if let Some(clk) = clk {
                map.insert(*epoch, *clk);
            }
        }
        map
    }

    /// Extracts distant clock offsets 
    /// (also refered to as "clock biases") in [s],
    /// on an epoch basis and per space vehicule,
    /// from this Navigation record.
    /// This does not produce anything if self is not a NAV RINEX.
    /// Use this to process [pseudo_range_to_distance]
    ///
    /// Example:
    /// ```
    /// use rinex::*;
    /// use rinex::sv::Sv;
    /// use rinex::constellation::Constellation;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx");
    /// let mut rinex = rinex.unwrap();
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
    /// let mut offsets = rinex
    ///     .space_vehicules_clock_offset();
    /// // example: apply a static offset to all clock offsets
    /// for (e, sv) in offsets.iter_mut() { // (epoch, vehicules)
    ///     for (sv, offset) in sv.iter_mut() { // vehicule, clk_offset
    ///         *offset += 10.0_f64 // do something..
    ///     }
    /// }
    /// 
    /// // use these distant clock offsets,
    /// // to convert pseudo ranges to real distances,
    /// // in an associated OBS data set
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx");
    /// let mut rinex = rinex.unwrap();
    /// // apply same filter, we're still only interested in G07 + G08
    /// rinex
    ///     .retain_space_vehicule_mut(filter.clone());
    /// // apply conversion
    /// let distances = rinex.observation_pseudo_range_to_distance(offsets);
    /// ```
    pub fn space_vehicules_clock_offset (&self) 
            -> BTreeMap<Epoch, BTreeMap<Sv, f64>> 
    {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to extract
        }
        let mut results: BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
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
    /// use rinex::*;
    /// use rinex::sv::Sv;
    /// use rinex::constellation::Constellation;
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx");
    /// let mut rinex = rinex.unwrap();
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
    /// let mut biases = rinex.space_vehicules_clock_biases();
    /// // example: adjust clock offsets and drifts
    /// for (e, sv) in biases.iter_mut() { // (epoch, vehicules)
    ///     for (sv, (offset, dr, drr)) in sv.iter_mut() { // vehicule, (offset, drift, drift/dt)
    ///         *offset += 10.0_f64; // do something..
    ///         *dr = dr.powf(0.25); // do something..
    ///     }
    /// }
    /// ```
    pub fn space_vehicules_clock_biases (&self) 
            -> BTreeMap<Epoch, BTreeMap<Sv, (f64,f64,f64)>> 
    {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to extract
        }
        let mut results: BTreeMap<Epoch, BTreeMap<Sv, (f64,f64,f64)>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::Ephemeris {
                    let mut map :BTreeMap<Sv, (f64,f64,f64)> = BTreeMap::new();
                    for frame in frames.iter() {
                        let (_, sv, ephemeris) = frame.as_eph().unwrap();
                        map.insert(*sv, 
							(ephemeris.clock_bias, 
							ephemeris.clock_drift,
							ephemeris.clock_drift_rate));
                    }
                    if map.len() > 0 { // got something
                        results.insert(*e, map);
                    }
                }
            }
        }
        results
    }

    /// Computes average epoch duration of this record
    pub fn average_epoch_duration (&self) -> Result<chrono::Duration, time::OutOfRangeError> {
        let mut sum_secs = 0_u64;
        //let mut sum_nanos = 0;
        let epochs = self.epochs();
        for i in 1..epochs.len() {
            let dt = epochs[i].date - epochs[i-1].date;
            sum_secs += dt.num_seconds() as u64;
        }
        let duration = std::time::Duration::from_secs(sum_secs / epochs.len() as u64);
        chrono::Duration::from_std(duration)
    }

    /// Returns list of observables, in the form 
    /// of standardized 3 letter codes, that can be found in this record.
    /// This does not produce anything in case of ATX and IONEX records.
    /// In case of NAV record:
    ///    - Ephemeris frames: returns list of Orbits identifier,
	///    example: "iode", "cus"..
    ///    - System Time Offset frames: list of Time Systems ("GAUT", "GAGP"..)
    ///    - Ionospheric Models: does not apply
    pub fn observables (&self) -> Vec<String> {
        let mut result :Vec<String> = Vec::new();
        if let Some(obs) = &self.header.obs {
            for (constell, codes) in obs.codes.iter() {
                for code in codes {
                    result.push(format!("{}:{}", 
                        constell.to_3_letter_code(),
                        code.to_string()))
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
            let record = self.record
                .as_nav()
                .unwrap();
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
    pub fn space_vehicules_per_epoch (&self) -> BTreeMap<Epoch, Vec<Sv>> {
        let mut map: BTreeMap<Epoch, Vec<Sv>> = BTreeMap::new();
        if self.is_navigation_rinex() {
            let record = self.record
                .as_nav()
                .unwrap();
            for (epoch, classes) in record.iter() {
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
                    map.insert(*epoch, inner);
                }
            }

        } else if self.is_observation_rinex() {
            let record = self.record
                .as_obs()
                .unwrap();
            for (epoch, (_, vehicules)) in record.iter() {
                let mut inner: Vec<Sv> = Vec::new();
                for (sv, _) in vehicules.iter() {
                    inner.push(*sv);
                }
                if inner.len() > 0 {
                    map.insert(*epoch, inner);
                }
            }
        }
        map
    }

    /// Returns list of space vehicules encountered in this file
    pub fn space_vehicules (&self) -> Vec<Sv> {
        let mut map: Vec<sv::Sv> = Vec::new();
        if self.is_navigation_rinex() {
            let record = self.record
                .as_nav()
                .unwrap();
            for (_, classes) in record.iter() {
                for (class, frames) in classes.iter() {
                    if *class == navigation::FrameClass::Ephemeris {
                        for frame in frames {
                            let (_, sv, _) = frame.as_eph().unwrap();
                            if !map.contains(&sv) {
                                map.push(sv.clone());
                            }
                        }
                    }
                }
            }

        } else if self.is_observation_rinex() {
            let record = self.record
                .as_obs()
                .unwrap();
            for (_, (_, vehicules)) in record.iter() {
                for (sv, _) in vehicules.iter() {
                    if !map.contains(&sv) {
                        map.push(sv.clone());
                    }
                }
            }
        }
        map.sort();
        map
    }

    /// List systems contained in this Clocks RINEX,
    /// system can either a station or a space vehicule
    pub fn list_clock_systems (&self) -> Vec<clocks::record::System> {
        let mut map: Vec<clocks::record::System> = Vec::new();
        if !self.is_clocks_rinex() {
            return map;
        }
        let record = self.record
            .as_clock()
            .unwrap();
        for (_, dtypes) in record.iter() {
            for (_dtype, systems) in dtypes.iter() {
                for (system, _) in systems.iter() {
                    if !map.contains(&system) {
                        map.push(system.clone())
                    }
                }
            }
        }
        map.sort();
        map
    }

    /// Lists systems contained in this Clocks RINEX,
    /// on an epoch basis. Systems can either be a station or a space vehicule.
    pub fn clock_systems_per_epoch (&self) -> BTreeMap<Epoch, Vec<clocks::record::System>> {
        let mut map: BTreeMap<Epoch, Vec<clocks::record::System>> = BTreeMap::new();
        if !self.is_clocks_rinex() {
            return map;
        }
        let record = self.record
            .as_clock()
            .unwrap();
        for (epoch, dtypes) in record.iter() {
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
        map
    }

    /// Filters out data records that do not contained in the given Observable list. 
    /// For Observation record: "C1C", "L1C", ..., any valid 3 letter observable.
    /// This only applies to Meteo Observations.
    /// Observation record example:
    /// ```
    /// use rinex::*;
    /// let mut rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O").unwrap();
    /// rinex
    ///     .retain_observable_mut(vec!["C1C","C2P"]);
    /// ```
    pub fn retain_observable_mut (&mut self, filter: Vec<&str>) {
        if self.is_observation_rinex() {
            let record = self.record
                .as_mut_obs()
                .unwrap();
            record
                .retain(|_, (_clk, vehicules)| {
                    vehicules.retain(|_, obs| {
                        obs.retain(|code, _| {
                            filter.contains(&code.as_str())
                        });
                        obs.len() > 0
                    });
                    vehicules.len() > 0
                });
        } else if self.is_meteo_rinex() {
            let record = self.record
                .as_mut_meteo()
                .unwrap();
            record
                .retain(|_, data| {
                    data.retain(|code, _| {
                        filter.contains(&code.to_string().as_str())
                    });
                    data.len() > 0
                });
        }
    }

    /// Immutable implementation of [retain_observable_mut]
    pub fn retain_observable (&self, filter: Vec<&str>) -> Self {
        let mut s = self.clone();
        s.retain_observable_mut(filter);
        s
    }
    
    /// Filters out Ephemeris Orbits data we are not interested in.
	/// Also filters out non ephemeris data.
    /// This has no effect over non Navigation RINEX.
    /// Example:
    /// ```
    /// use rinex::*;
    /// let mut rnx = Rinex::from_file("../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx").unwrap();
    /// rnx
    ///     .retain_ephemeris_orbits_mut(vec!["satPosX","satPosY","satPosZ"]);
    /// ```
    pub fn retain_ephemeris_orbits_mut (&mut self, filter: Vec<&str>) {
        if !self.is_navigation_rinex() {
            return ;
        }
        let record = self.record
            .as_mut_nav()
            .unwrap();
        record.retain(|_, classes| {
            classes.retain(|class, frames| {
                if *class == navigation::FrameClass::Ephemeris {
                    frames.retain_mut(|fr| {
                        let (_, _, ephemeris) = fr.as_mut_eph().unwrap();
						let orbits = &mut ephemeris.orbits;
                        orbits.retain(|k,_| filter.contains(&k.as_str()));
                        orbits.len() > 0
                    });
                    frames.len() > 0
                } else {
                    false
                }
            });
            classes.len() > 0
        });
    }

    /// Immutable implementation of [retain_ephemeris_orbits_mut]
    pub fn retain_ephemeris_orbits_filter (&self, filter: Vec<&str>) -> Self {
        let mut s = self.clone();
        s.retain_ephemeris_orbits_mut(filter);
        s
    }

	/// Retains Navigation messages contained in given list.
	/// An example of MsgType is LNAV for legacy frames.
	pub fn retain_navigation_message_mut (&mut self, filter: Vec<navigation::MsgType>) {
		if !self.is_navigation_rinex() {
			return ;
		}
		let record = self.record
			.as_mut_nav()
			.unwrap();
		record.retain(|_, classes| {
			classes.retain(|class, frames| {
				if *class == navigation::FrameClass::Ephemeris {
					frames.retain(|fr| {
						let (msg, _, _) = fr.as_eph()
							.unwrap();
						filter.contains(&msg)
					});
				} else if *class == navigation::FrameClass::SystemTimeOffset {
					frames.retain(|fr| {
						let (msg, _, _) = fr.as_sto()
							.unwrap();
						filter.contains(&msg)
					});
				} else if *class == navigation::FrameClass::IonosphericModel {
					frames.retain(|fr| {
						let (msg, _, _) = fr.as_ion()
							.unwrap();
						filter.contains(&msg)
					});
				} else { 
					frames.retain(|fr| {
						let (msg, _, _) = fr.as_eop()
							.unwrap();
						filter.contains(&msg)
					});
				}
				frames.len() > 0
			});
			classes.len() > 0
		});
	}
	
	/// Immutable implementation, see [retain_navigation_message_mut]
	pub fn retain_navigation_message (&self, filter: Vec<navigation::MsgType>) -> Self {
		let mut s = self.clone();
		s.retain_navigation_message_mut(filter);
		s
	}

    /// Retains only Navigation frames that are marked as legacy.
	/// Retains only Ephemeris in case of V2/V3 RINEX.
	/// Retains all kinds of legacy frames in case of V4 RINEX.
    /// This has no effect if self is not a Navigation record.
	pub fn retain_legacy_navigation_mut (&mut self) {
		self.retain_navigation_message_mut(vec![navigation::MsgType::LNAV])
	}
       
	/// Immutable implementation, see [retain_legacy_navigation_mut]
	pub fn retain_legacy_navigation(&self) -> Self {
		let mut s = self.clone();
        s.retain_legacy_navigation_mut();
        s
	}

    /// Retains only modern Navigation frames.
    /// This has no effect if self is not a Navigation record.
    pub fn retain_modern_navigation_mut(&mut self) {
		self.retain_navigation_message_mut(vec![
			navigation::MsgType::FDMA,
			navigation::MsgType::IFNV,
			navigation::MsgType::D1,
			navigation::MsgType::D2,
			navigation::MsgType::D1D2,
			navigation::MsgType::SBAS,
			navigation::MsgType::CNVX,
		])
    }

    /// Immutable implementation
    pub fn retain_modern_navigation (&self) -> Self {
        let mut s = self.clone();
        s.retain_modern_navigation_mut();
        s
    }

    /// Applies given AND mask in place, to all observations.
    /// This has no effect on non observation records.
    /// This also drops observations that did not come with an LLI flag
    pub fn observation_lli_and_mask_mut (&mut self, mask: observation::LliFlags) {
        if !self.is_observation_rinex() {
            return ; // nothing to browse
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
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

    /// Retains data with a minimum SSI Signal Strength requirement.
    /// All observation that do not match the |s| > ssi (excluded) predicate,
    /// get thrown away. All observation that did not come with an SSI attached
    /// to them get thrown away too (can't make a decision).
    /// This can act as a simple signal quality filter.
    /// This has no effect on non Observation Data.
    pub fn minimum_sig_strength_filter_mut (&mut self, minimum: observation::Ssi) {
        if !self.is_observation_rinex() {
            return ; // nothing to browse
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record
            .retain(|_, (_clk, vehicules)| {
                vehicules.retain(|_, obs| {
                    obs.retain(|_, data| {
                        if let Some(ssi) = data.ssi {
                            ssi > minimum
                        } else {
                            false // no ssi: drop out
                        }
                    });
                    obs.len() > 0
                });
                vehicules.len() > 0
            });
    }

    /// Immutable implementation of [minimum_sig_strength_filter_mut]
    pub fn minimum_sig_strength_filter (&self, minimum: observation::Ssi) -> Self {
        let mut filtered = self.clone();
        filtered
            .minimum_sig_strength_filter_mut(minimum);
        filtered
    }

    /// Extracts signal strength as (min, max) duplet,
    /// accross all vehicules.
    /// Only relevant on Observation RINEX.
    pub fn sig_strength_min_max (&self) -> Option<(observation::Ssi, observation::Ssi)> {
        if self.is_observation_rinex() {
            let (mut min, mut max) = (observation::Ssi::DbHz54, observation::Ssi::DbHz0);
            let record = self.record
                .as_obs()
                .unwrap();
            for (_, (_, vehicules)) in record.iter() {
                for (_, observation) in vehicules.iter() {
                    for (_, data) in observation.iter() {
                        if let Some(ssi) = data.ssi {
                            if ssi < min {
                                min = ssi
                            } else if ssi > max {
                                max = ssi
                            }
                        }
                    }
                }
            }
            Some((min,max))
        } else {
            None
        }
    }

    /// Extracts signal strength as (min, max) duplet,
    /// per vehicule. Only relevant on Observation RINEX
    pub fn sig_strength_sv_min_max (&self) -> HashMap<Sv, (observation::Ssi, observation::Ssi)> {
        let mut map: HashMap<Sv, (observation::Ssi, observation::Ssi)>
            = HashMap::new();
        if let Some(record) = self.record.as_obs() {
            for (_, (_, vehicules)) in record.iter() {
                for (sv, observations) in vehicules.iter() {
                    let (mut min, mut max) = (observation::Ssi::DbHz54, observation::Ssi::DbHz0);
                    for (_, observation) in observations.iter() {
                        if let Some(ssi) = observation.ssi {
                            min = std::cmp::min(min, ssi);
                            max = std::cmp::max(max, ssi);
                        }
                    }
                    map.insert(*sv, (min,max));
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
    pub fn ephemeris (&self) -> BTreeMap<Epoch, BTreeMap<sv::Sv, (f64,f64,f64, HashMap<String, OrbitItem>)>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new() ; // nothing to browse
        }
        let mut results: BTreeMap<Epoch, BTreeMap<sv::Sv, (f64,f64,f64, HashMap<String, OrbitItem>)>>
            = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::Ephemeris {
                    let mut inner: BTreeMap<sv::Sv,  (f64,f64,f64, HashMap<String, OrbitItem>)> = BTreeMap::new();
                    for frame in frames.iter() {
                        let (_, sv, ephemeris) = frame.as_eph().unwrap();
                        inner.insert(*sv, 
							(ephemeris.clock_bias,
							ephemeris.clock_drift, 
							ephemeris.clock_drift_rate, 
							ephemeris.orbits.clone()));
                    }
                    if inner.len() > 0 {
                        results.insert(*e, inner);
                    }
                }
            }
        }
        results
    }

    /// Extracts elevation angle for all vehicules per epoch,
    /// Does not produce anything if this is not an NAV record,
    /// or NAV record does not contain any ephemeris frame
    pub fn orbits_elevation_angles (&self) -> BTreeMap<Epoch, BTreeMap<Sv, f64>> {
        let mut ret : BTreeMap<Epoch, BTreeMap<Sv, f64>> = BTreeMap::new();
        if !self.is_navigation_rinex() {
            return ret ;
        }
        let record = self.record
            .as_nav()
            .unwrap();
        for (epoch, classes) in record.iter() {
            let mut inner: BTreeMap<sv::Sv, f64> = BTreeMap::new();
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::Ephemeris {
                    for frame in frames.iter() {
                        let (_, sv, ephemeris) = frame.as_eph()
                            .unwrap();
						let orbits = &ephemeris.orbits;
                        // test all well known elevation mask fields
                        if let Some(elev) = orbits.get("e") {
                            inner.insert(sv.clone(), elev.as_f64().unwrap());
                        } else {
                            if let Some(posx) = orbits.get("satPosX") {
                                if let Some(posy) = orbits.get("satPosY") {
                                    if let Some(posz) = orbits.get("satPosY") {
                                        let e = posx.as_f64().unwrap() * posy.as_f64().unwrap() * posz.as_f64().unwrap(); //TODO
                                        inner.insert(sv.clone(), e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if inner.len() > 0 {
                ret.insert(*epoch, inner);
            }
        }
        ret
    }

    /// Filters out all vehicules that exhibit an elevation angle below given mask (a < min_angle).
    /// Has no effect if self is not a NAV record containing at least 1 Ephemeris frame.
    pub fn orbits_elevation_angle_filter_mut (&mut self, min_angle: f64) {
        if !self.is_navigation_rinex() {
            return ;
        }
        let record = self.record
            .as_mut_nav()
            .unwrap();
        record
            .retain(|_, classes| {
                classes.retain(|class, frames| {
                    if *class == navigation::FrameClass::Ephemeris {
                        frames.retain(|fr| {
                            let (_, _, ephemeris) = fr.as_eph()
                                .unwrap();
                            if let Some(elev) = ephemeris.orbits.get("e") {
                                elev.as_f64().unwrap() < min_angle
                            } else { // TODO
                                false
                            }
                        });
                        frames.len() > 0 
                    } else { // not an EPH
                        true // keep it anyway
                    }
                });
                classes.len() > 0
            })
    }

    /// Immutable implementation of [elevation_angle_filter_mut]
    pub fn orbits_elevation_angle_filter (&self, min_angle: f64) -> Self {
        let mut s = self.clone();
        s.orbits_elevation_angle_filter_mut(min_angle);
        s
    }

    /// Filters out vehicules for each epoch where they did not exhibit
    /// an elevation angle that is contained in (min_angle <= a <= max_angle)
    /// both included.
    /// This has no effect on RINEX records other than NAV records
    pub fn orbits_elevation_angle_range_filter_mut (&mut self, min_max: (f64,f64)) {
        if !self.is_navigation_rinex() {
            return ;
        }
        let (min, max) = min_max;
        let record = self.record
            .as_mut_nav()
            .unwrap();
        record.retain(|_, classes| {
            classes.retain(|_, frames| {
                frames.retain(|fr| {
                    let (_, _, ephemeris) = fr.as_eph().unwrap();
                    if let Some(elev) = ephemeris.orbits.get("e") {
                        let elev = elev.as_f64().unwrap();
                        elev >= min && elev <= max 
                    } else { //TODO do other keys exist? what about GLO?
                        false
                    }
                });
                frames.len() > 0
            });
            classes.len() > 0
        })
    }

    /// Immutable implementation of [elevation_angle_interval_mut]
    pub fn orbits_elevation_angle_range_filter (&self, min_max: (f64,f64)) -> Self {
        let mut s = self.clone();
        s.orbits_elevation_angle_range_filter_mut(min_max);
        s
    }

    /// Extracts all System Time Offset data
    /// on a epoch basis, from this Navigation record.
    /// This does not produce anything if self is not a modern Navigation record
    /// that contains such frames.
    pub fn navigation_system_time_offsets (&self) -> BTreeMap<Epoch, Vec<navigation::StoMessage>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::StoMessage>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::SystemTimeOffset {
                    let mut inner :Vec<navigation::StoMessage> = Vec::new();
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
    pub fn navigation_ionospheric_models (&self) -> BTreeMap<Epoch, Vec<navigation::IonMessage>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new(); // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::IonMessage>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner :Vec<navigation::IonMessage> = Vec::new();
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
    pub fn navigation_klobuchar_ionospheric_models (&self) -> BTreeMap<Epoch, Vec<navigation::KbModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new() ; // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::KbModel>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner :Vec<navigation::KbModel> = Vec::new();
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
    pub fn navigation_nequickg_ionospheric_models (&self) -> BTreeMap<Epoch, Vec<navigation::NgModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new() ; // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::NgModel>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner :Vec<navigation::NgModel> = Vec::new();
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
    pub fn navigation_bdgim_ionospheric_models (&self) -> BTreeMap<Epoch, Vec<navigation::BdModel>> {
        if !self.is_navigation_rinex() {
            return BTreeMap::new() ; // nothing to browse
        }
        let mut results: BTreeMap<Epoch, Vec<navigation::BdModel>> = BTreeMap::new();
        let record = self.record
            .as_nav()
            .unwrap();
        for (e, classes) in record.iter() {
            for (class, frames) in classes.iter() {
                if *class == navigation::FrameClass::IonosphericModel {
                    let mut inner :Vec<navigation::BdModel> = Vec::new();
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

    /// Extracts Pseudo Range data from this
    /// Observation record, on an epoch basis an per space vehicule. 
    /// Does not produce anything if self is not an Observation RINEX.
    pub fn observation_pseudo_ranges (&self) -> BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> {
        let mut results: BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> = BTreeMap::new();
        if !self.is_observation_rinex() {
            return results ; // nothing to browse
        }
        let record = self.record
            .as_obs()
            .unwrap();
        for (e, (_, sv)) in record.iter() {
            let mut map: BTreeMap<sv::Sv, Vec<(String, f64)>> = BTreeMap::new();
            for (sv, obs) in sv.iter() {
                let mut v : Vec<(String, f64)> = Vec::new();
                for (code, data) in obs.iter() {
                    if is_pseudo_range_obs_code!(code) {
                        v.push((code.clone(), data.obs));
                    }
                }
                if v.len() > 0 { // did come with at least 1 PR
                    map.insert(*sv, v);
                }
            }
            if map.len() > 0 { // did produce something
                results.insert(*e, map);
            }
        }
        results
    }
    
    /// Extracts Pseudo Ranges without Ionospheric path delay contributions,
    /// by extracting [pseudo_ranges] and using the differential (dual frequency) compensation.
    /// We can only compute such information if pseudo range was evaluted
    /// on at least two seperate carrier frequencies, for a given space vehicule at a certain epoch.
    /// Does not produce anything if self is not an Observation RINEX.
    pub fn iono_free_pseudo_ranges (&self) -> BTreeMap<Epoch, BTreeMap<sv::Sv, f64>> {
        let pr = self.observation_pseudo_ranges();
        let mut results : BTreeMap<Epoch, BTreeMap<sv::Sv, f64>> = BTreeMap::new();
        for (e, sv) in pr.iter() {
            let mut map :BTreeMap<sv::Sv, f64> = BTreeMap::new();
            for (sv, obs) in sv.iter() {
                let mut result :Option<f64> = None; 
                let mut retained : Vec<(String, f64)> = Vec::new();
                for (code, value) in obs.iter() {
                    if is_pseudo_range_obs_code!(code) {
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
                    let mut channels :Vec<channel::Channel> = Vec::with_capacity(2);
                    for i in 0..codes.len() {
                        if let Ok(channel) = channel::Channel::from_observable(sv.constellation, &codes[i]) {
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
    
    /// Extracts Raw Carrier Phase observations,
    /// from this Observation record, on an epoch basis an per space vehicule. 
    /// Does not produce anything if self is not an Observation RINEX.
    pub fn observation_carrier_phases (&self) -> BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> {
        let mut results: BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> = BTreeMap::new();
        if !self.is_observation_rinex() {
            return results ; // nothing to browse
        }
        let record = self.record
            .as_obs()
            .unwrap();
        for (e, (_, sv)) in record.iter() {
            let mut map: BTreeMap<sv::Sv, Vec<(String, f64)>> = BTreeMap::new();
            for (sv, obs) in sv.iter() {
                let mut v : Vec<(String, f64)> = Vec::new();
                for (code, data) in obs.iter() {
                    if is_phase_carrier_obs_code!(code) {
                        v.push((code.clone(), data.obs));
                    }
                }
                if v.len() > 0 { // did come with at least 1 Phase obs
                    map.insert(*sv, v);
                }
            }
            if map.len() > 0 { // did produce something
                results.insert(*e, map);
            }
        }
        results
    }
    
    /// Extracts Carrier phases without Ionospheric path delay contributions,
    /// by extracting [carrier_phases] and using the differential (dual frequency) compensation.
    /// We can only compute such information if carrier phase was evaluted
    /// on at least two seperate carrier frequencies, for a given space vehicule at a certain epoch.
    /// Does not produce anything if self is not an Observation RINEX.
    pub fn iono_free_observation_carrier_phases (&self) -> BTreeMap<Epoch, BTreeMap<sv::Sv, f64>> {
        let pr = self.observation_pseudo_ranges();
        let mut results : BTreeMap<Epoch, BTreeMap<sv::Sv, f64>> = BTreeMap::new();
        for (e, sv) in pr.iter() {
            let mut map :BTreeMap<sv::Sv, f64> = BTreeMap::new();
            for (sv, obs) in sv.iter() {
                let mut result :Option<f64> = None; 
                let mut retained : Vec<(String, f64)> = Vec::new();
                for (code, value) in obs.iter() {
                    if is_phase_carrier_obs_code!(code) {
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
                    let mut channels :Vec<channel::Channel> = Vec::with_capacity(2);
                    for i in 0..codes.len() {
                        if let Ok(channel) = channel::Channel::from_observable(sv.constellation, &codes[i]) {
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
    /// use rinex::*;
    /// use rinex::sv::Sv;
    /// use rinex::constellation::Constellation;
    /// // obtain distance clock offsets, by analyzing a related NAV file
    /// // (this is only an example..)
    /// let rinex = Rinex::from_file("../test_resources/NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx");
    /// let mut rinex = rinex.unwrap();
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
    /// let distances = rinex.observation_pseudo_range_to_distance(sv_clk_offsets);
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
    pub fn observation_pseudo_range_to_distance (&self, sv_clk_offsets: BTreeMap<Epoch, BTreeMap<sv::Sv, f64>>) -> BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> {
        let mut results :BTreeMap<Epoch, BTreeMap<sv::Sv, Vec<(String, f64)>>> = BTreeMap::new();
        if !self.is_observation_rinex() {
            return results ;
        }
        let record = self.record
            .as_obs()
            .unwrap();
        for (e, (clk, sv)) in record.iter() {
            if let Some(distant_e) = sv_clk_offsets.get(e) { // got related distant epoch
                if let Some(clk) = clk { // got local clock offset 
                    let mut map : BTreeMap<sv::Sv, Vec<(String, f64)>> = BTreeMap::new();
                    for (sv, obs) in sv.iter() {
                        if let Some(sv_offset) = distant_e.get(sv) { // got related distant offset
                            let mut v : Vec<(String, f64)> = Vec::new();
                            for (code, data) in obs.iter() {
                                if is_pseudo_range_obs_code!(code) {
                                    // We currently do not support the compensation for biases
                                    // than clock induced ones. ie., Ionospheric delays ??
                                    v.push((code.clone(), data.pr_real_distance(*clk, *sv_offset, 0.0)));
                                }
                            }
                            if v.len() > 0 { // did come with at least 1 PR
                                map.insert(*sv, v);
                            }
                        } // got related distant offset
                    } // per sv
                    if map.len() > 0 { // did produce something
                        results.insert(*e, map);
                    }
                } // got local clock offset attached to this epoch
            }//got related distance epoch
        } // per epoch
        results
    }

    /// Decimates record to fit minimum required epoch interval.
    /// All epochs that do not match the requirement
    /// |e(k).date - e(k-1).date| < interval, get thrown away.
    /// Also note we adjust the INTERVAL field,
    /// meaning, further file production will be correct.
    pub fn decimate_by_interval_mut (&mut self, interval: chrono::Duration) {
        let mut last_preserved = self.epochs()[0].date;
        match self.header.rinex_type {
            types::Type::NavigationData => {
                let record = self.record
                    .as_mut_nav()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
            },
            types::Type::ObservationData => {
                let record = self.record
                    .as_mut_obs()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
            },
            types::Type::MeteoData => {
                let record = self.record
                    .as_mut_meteo()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
            },
            types::Type::ClockData => {
                let record = self.record
                    .as_mut_clock()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
            },
            types::Type::IonosphereMaps => {
                let record = self.record
                    .as_mut_ionex()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
            },
            _ => panic!("operation is not feasible"),
        }
    }

    /// Refer to [decimate_by_interval], non mutable implementation
    pub fn decimate_by_interval (&self, interval: chrono::Duration) -> Self {
        let mut last_preserved = self.epochs()[0].date;
        let record: record::Record = match self.header.rinex_type {
            types::Type::NavigationData => {
                let mut record = self.record
                    .as_nav()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
                record::Record::NavRecord(record)
            },
            types::Type::ObservationData => {
                let mut record = self.record
                    .as_obs()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
                record::Record::ObsRecord(record)
            },
            types::Type::MeteoData => {
                let mut record = self.record
                    .as_meteo()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
                record::Record::MeteoRecord(record)
            },
            types::Type::IonosphereMaps => {
                let mut record = self.record
                    .as_ionex()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
                record::Record::IonexRecord(record)
            },
            types::Type::ClockData => {
                let mut record = self.record
                    .as_clock()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = e.date - last_preserved;
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= interval {
                            last_preserved = e.date;
                            true
                        } else {
                            false
                        }
                    } else {
                        last_preserved = e.date;
                        true
                    }
                });
                record::Record::ClockRecord(record)
            },
            _ => panic!("ops is not feasible"),
        };
        Self {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record,
        }
    }
    
    /// Decimates (reduce record quantity) by given ratio.
    /// For example, ratio = 2, we keep one out of two entry,
    /// regardless of epoch interval and interval values.
    /// This works on any time of record, since we do not care,
    /// about the internal information, just the number of entries in the record. 
    pub fn decimate_by_ratio_mut (&mut self, ratio: u32) {
        let mut counter = 0;
        match self.header.rinex_type {
            types::Type::NavigationData => {
                let record = self.record
                    .as_mut_nav()
                    .unwrap();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
            types::Type::ObservationData => {
                let record = self.record
                    .as_mut_obs()
                    .unwrap();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
            types::Type::MeteoData => {
                let record = self.record
                    .as_mut_meteo()
                    .unwrap();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
            types::Type::ClockData => {
                let record = self.record
                    .as_mut_clock()
                    .unwrap();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
            types::Type::IonosphereMaps => {
                let record = self.record
                    .as_mut_ionex()
                    .unwrap();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
            types::Type::AntennaData => {
                let record = self.record
                    .as_mut_antex()
                    .unwrap();
                record.retain(|_| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
            },
        }
    }

    /// See [decimate_by_ratio_mut]
    pub fn decimate_by_ratio (&self, ratio: u32) -> Self {
        let mut counter = 0;
        let record :record::Record = match self.header.rinex_type {
            types::Type::NavigationData => {
                let mut record = self.record
                    .as_nav()
                    .unwrap()
                    .clone();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
                record::Record::NavRecord(record)
            },
            types::Type::ObservationData => {
                let mut record = self.record
                    .as_obs()
                    .unwrap()
                    .clone();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
                record::Record::ObsRecord(record)
            },
            types::Type::MeteoData => {
                let mut record = self.record
                    .as_meteo()
                    .unwrap()
                    .clone();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
                record::Record::MeteoRecord(record)
            },
            types::Type::IonosphereMaps => {
                let mut record = self.record
                    .as_ionex()
                    .unwrap()
                    .clone();
                record.retain(|_, _| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
                record::Record::IonexRecord(record)
            },
            types::Type::AntennaData => {
                let mut record = self.record
                    .as_antex()
                    .unwrap()
                    .clone();
                record.retain(|_| {
                    let retain = (counter % ratio) == 0;
                    counter += 1;
                    retain
                });
                record::Record::AntexRecord(record)
            },
            _ => todo!("implement other record types"),
        };
        Self {
            header: self.header.clone(),
            comments: self.comments.clone(),
            record,
        }
    }
/*
    /// "Upsamples" to match desired epoch interval
    pub fn upsample_by_interval_mut (&mut self, interval: chrono::Duration) {
        let epochs = self.epochs();
        match self.header.rinex_type {
            types::Type::NavigationData => {
                let record = self.record
                    .as_mut_nav()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter() {
                    let n = (e.date - prev_epoch.date).num_seconds() / interval.num_seconds(); // nb of epoch to insert
                    for i in 0..n {
                        record.insert(
                            Epoch {
                                date: e.date + chrono::Duration {
                                    secs: i * interval.num_seconds(),
                                    nanos: 0,
                                },
                                flag: e.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::ObservationData => {
                let record = self.record
                    .as_mut_obs()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter() {
                    let n = (e.date - prev_epoch.date).num_seconds() / interval.num_seconds(); // nb of epoch to insert
                    for i in 0..n {
                        record.insert(
                            Epoch {
                                date: e.date + chrono::Duration {
                                    secs: i * interval.num_seconds(),
                                    nanos: 0,
                                },
                                flag: e.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::MeteoData => {
                let record = self.record
                    .as_mut_meteo()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter() {
                    let n = (e.date - prev_epoch.date).num_seconds() / interval.num_seconds(); // nb of epoch to insert
                    for i in 0..n {
                        record.insert(
                            Epoch {
                                date: e.date + chrono::Duration {
                                    secs: i * interval.num_seconds(),
                                    nanos: 0,
                                },
                                flag: e.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::ClockData => {
                let record = self.record
                    .as_mut_clock()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter() {
                    let n = (e.date - prev_epoch.date).num_seconds() / interval.num_seconds(); // nb of epoch to insert
                    for i in 0..n {
                        record.insert(
                            Epoch {
                                date: e.date + chrono::Duration {
                                    secs: i * interval.num_seconds(),
                                    nanos: 0,
                                },
                                flag: e.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::IonosphereMaps => {
                let record = self.record
                    .as_mut_ionex()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter() {
                    let n = (e.date - prev_epoch.date).num_seconds() / interval.num_seconds(); // nb of epoch to insert
                    for i in 0..n {
                        record.insert(
                            Epoch {
                                date: e.date + chrono::Duration {
                                    secs: i * interval.num_seconds(),
                                    nanos: 0,
                                },
                                flag: e.flag,
                            },
                            data.clone());
                    }
                }
            },
            _ => {},
        }
    }

    /// "Upsamples" self by given ratio by copying each epoch
    /// ratio-1 times. Let say self had 10 epochs and ratio is 2, we copy all epochs
    /// once. There is no filtering operation involved, just plain copies.
    pub fn upsample_by_ratio_mut (&mut self, ratio: u32) {
        let epochs = self.epochs();
        match self.header.rinex_type {
            types::Type::NavigationData => {
                let record = self.record
                    .as_mut_nav()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, classes) in record.iter().skip(1) {
                    let dt = (e.date - prev_epoch.date) / ratio as i32;
                    for j in 1..ratio {
                        record.insert(
                            Epoch {
                                date: prev_epoch.date +dt*j as i32,
                                flag: prev_epoch.flag,
                            },
                            classes.clone());    
                    }
                }
            },
            types::Type::ObservationData => {
                let record = self.record
                    .as_mut_obs()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter().skip(1) {
                    let dt = (e.date - prev_epoch.date) / ratio as i32;
                    for j in 1..ratio {
                        record.insert(
                            Epoch {
                                date: prev_epoch.date +dt *j as i32,
                                flag: prev_epoch.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::MeteoData => {
                let record = self.record
                    .as_mut_meteo()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter().skip(1) {
                    let dt = (e.date - prev_epoch.date) / ratio as i32;
                    for j in 1..ratio {
                        record.insert(
                            Epoch {
                                date: prev_epoch.date +dt *j as i32,
                                flag: prev_epoch.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::ClockData => {
                let record = self.record
                    .as_mut_clock()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter().skip(1) {
                    let dt = (e.date - prev_epoch.date) / ratio as i32;
                    for j in 1..ratio {
                        record.insert(
                            Epoch {
                                date: prev_epoch.date +dt *j as i32,
                                flag: prev_epoch.flag,
                            },
                            data.clone());
                    }
                }
            },
            types::Type::IonosphereMaps => {
                let record = self.record
                    .as_mut_ionex()
                    .unwrap();
                let mut prev_epoch = epochs[0];
                for (e, data) in record.iter().skip(1) {
                    let dt = (e.date - prev_epoch.date) / ratio as i32;
                    for j in 1..ratio {
                        record.insert(
                            Epoch {
                                date: prev_epoch.date +dt *j as i32,
                                flag: prev_epoch.flag,
                            },
                            data.clone());
                    }
                }
            },
            _ => {} // does not apply, not epoch iterable
        }
    }
*/

    /// Computes the single difference between self and rhs Observation RINEX.
    /// This fails if both are not Observation RINEX files.
    /// This is intended to be used on shared epochs recorded through seperate devices,
    /// both ideally in stationnary position and held stationnary during entire measurement.
    /// Ideally, the same sampling period was used in both measurements.
    /// Only shared epochs and shared vehicules are retained in `self`!.
    /// We compute the difference between raw phase carrier data, other observables are preserved.
    /// The single difference minimizes the atmospheric and satellite vehicule
    /// induced biases.
    ///
    /// Example:
    /// ```
    /// use rinex::*;
    /// let mut rnx_a = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o").unwrap();
    /// let rnx_b = Rinex::from_file("../test_resources/OBS/V2/rovn0010.21o").unwrap();
    /// // compute the single difference, to cancel out ionospheric/atmospheric induced biases
    /// // rnx_a -= rnx_b; is actually feasible but not recommended,
    ///               // as it may panic of file type mismatch
    /// rnx_a
    ///     .observation_diff_mut(&rnx_b) // is recommended
    ///     .unwrap();
    ///
    /// // Remember only phase observables are modified, other observables are preserved. 
    /// // To access raw phase data, you then have two options
    /// // 1: [carrier_phases]
    /// let raw_phase = rnx_a.observation_carrier_phases();
    ///
    /// // 2: browsing `rnx_a` 
    /// let record = rnx_a.record.as_obs().unwrap();
    /// for (epoch, (_clk_offset, vehicules)) in record.iter() {
    ///     for (vehicule, obs) in vehicules.iter() {
    ///         for (obscode, data) in obs.iter() {
    ///             if is_phase_carrier_obs_code!(obscode) { // this is a phase observation
    ///                 // --> you know the differential op was applied to it
    ///                 let mut diff_phase = data.obs; // this is no low raw phase data
    ///                 diff_phase += std::f64::consts::PI; // do something with it
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn observation_diff_mut (&mut self, rhs: &Self) -> Result<(), DiffError> {
        if !self.is_observation_rinex() || !rhs.is_observation_rinex() {
            return Err(DiffError::NotObsRinex)
        }
        let (a, b) = (
            self.record
                .as_mut_obs()
                .unwrap(),
            rhs.record
                .as_obs()
                .unwrap());
        let b_epochs = rhs.epochs();
        a.retain(|e, _|  b_epochs.contains(e)); // retain shared epochs only!
        a.retain(|e, (_, vehicules)| { // retain shared vehicules only !
            if let Some((_, vvehicules)) = b.get(e) { 
                // shared epoch 
                vehicules.retain(|sv, _| { // drop non shared vehicules
                    vvehicules.get(sv).is_some()
                });
                vehicules.len() > 0 // at least 1 common vehicule
            } else { // no epoch in common
                false
            }
        });
        a.retain(|e, (_, vehicules)| { // retain shared observables, as far as Phase is concerned
            let (_, vehicules_b) = b.get(e)
                .unwrap(); // already filtered out
            vehicules.retain(|sv, observables| {
                let observables_b = vehicules_b.get(sv)
                    .unwrap(); // already filtered out
                observables.retain(|obscode, _| {
                    if is_phase_carrier_obs_code!(obscode) {
                        // this is phase observable: preserve if only shared by B
                        observables_b.get(obscode).is_some()
                    } else {
                        // not a phase observable: preserve by default
                        true
                    }
                });
                observables.len() > 0
            });
            vehicules.len() > 0
        });
        for (e, (_, vehicules_a)) in a.iter_mut() {
            let (_, vehicules_b) = b.get(e)
                .unwrap(); // already filtered out
            for (vehicule_a, obs_a) in vehicules_a.iter_mut() {
                let obs_b = vehicules_b.get(vehicule_a)
                    .unwrap(); // already filtered out
                for (obscode_a, data_a) in obs_a.iter_mut() {
                    if is_phase_carrier_obs_code!(obscode_a) {
                        let data_b = obs_b.get(obscode_a)
                            .unwrap(); // already filtered out
                        data_a.obs -= data_b.obs
                    }
                }
            }
        }
        Ok(())
    }

    /// See [observation_diff_mut], immutable implementation.
    pub fn observation_diff (&self, rhs: &Self) -> Result<Self, DiffError> {
        let mut s = self.clone();
        s.observation_diff_mut(rhs)?;
        Ok(s)
    }
    
    /// Restrain epochs to interval |start <= e <= end| (both included)
    pub fn time_window_mut (&mut self, start: chrono::NaiveDateTime, end: chrono::NaiveDateTime) {
        if self.is_observation_rinex() {
            let record = self.record
                .as_mut_obs()
                .unwrap();
            record
                .retain(|e, _| e.date >= start && e.date <= end);
        } else if self.is_navigation_rinex() {
            let record = self.record
                .as_mut_nav()
                .unwrap();
            record
                .retain(|e, _| e.date >= start && e.date <= end);
        } else if self.is_meteo_rinex() {
            let record = self.record
                .as_mut_meteo()
                .unwrap();
            record
                .retain(|e, _| e.date >= start && e.date <= end);
        } else if self.is_clocks_rinex() {
            let record = self.record
                .as_mut_clock()
                .unwrap();
            record
                .retain(|e, _| e.date >= start && e.date <= end);
        }
    }
    
    /// Returns a copy version of `self` where epochs were constrained
    /// to |start <= e <= end| interval (both included)
    pub fn time_window (&self, start: chrono::NaiveDateTime, end: chrono::NaiveDateTime) -> Self {
        let mut s = self.clone();
        s.time_window_mut(start, end);
        s
    }

    /// Returns list of epochs where cycles slips might have happened.
    /// This information is directly given by the receiver that produced
    /// this Observation RINEX. Does not produce anything if self
    /// is not an Observation RINEX. This information can only be confirmed 
    /// using the double difference post processing, refer to [confirmed_cycle_slips]
    pub fn observation_cycle_slip_epochs (&self) -> Vec<Epoch> {
        if !self.is_observation_rinex() {
            return Vec::new() ;
        }
        let retained = self
            .observation_lli_and_mask(observation::LliFlags::HALF_CYCLE_SLIP);
        retained.epochs()
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
    /// let rnx_b = Rinex::from_file("../test_resources/OBS/V2/rovn0010.21o").unwrap();
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
    pub fn clock_agency_retain_mut (&mut self, filter: Vec<&str>) {
        if !self.is_clocks_rinex() {
            return ; // does not apply
        }
        let record = self.record
            .as_mut_clock()
            .unwrap();
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
    pub fn to_file (&self, path: &str) -> Result<(), Error> {
		let mut writer = BufferedWriter::new(path)?;
        write!(writer, "{}", self.header.to_string())?;
        self.record
            .to_file(&self.header, &mut writer)?;
        Ok(())
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
        assert_eq!(is_pseudo_range_obs_code!("C1P"), true);
        assert_eq!(is_pseudo_range_obs_code!("P1P"), true);
        assert_eq!(is_pseudo_range_obs_code!("L1P"), false);
        assert_eq!(is_phase_carrier_obs_code!("L1P"), true);
        assert_eq!(is_phase_carrier_obs_code!("D1P"), false);
        assert_eq!(is_doppler_obs_code!("D1P"), true);
        assert_eq!(is_doppler_obs_code!("L1P"), false);
        assert_eq!(is_sig_strength_obs_code!("S1P"), true);
        assert_eq!(is_sig_strength_obs_code!("L1P"), false);
    }
    #[test]
    fn test_shared_methods() {
        let time = chrono::NaiveTime::from_str("00:00:00").unwrap();
        assert_eq!(hourly_session_str(time), "a");
        let time = chrono::NaiveTime::from_str("00:30:00").unwrap();
        assert_eq!(hourly_session_str(time), "a");
        let time = chrono::NaiveTime::from_str("23:30:00").unwrap();
        assert_eq!(hourly_session_str(time), "x");
    }
}
