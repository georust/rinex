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
pub mod epoch;
pub mod hardware;
pub mod hatanaka;
pub mod header;
pub mod ionosphere;
pub mod meteo;
pub mod navigation;
pub mod observation;
pub mod record;
pub mod sv;
pub mod types;
pub mod version;
pub mod reader;

use reader::BufferedReader;
use std::io::{Read, Write};

use thiserror::Error;
use chrono::{Datelike, Timelike};

#[cfg(feature = "with-serde")]
#[macro_use]
extern crate serde;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_comment {
    ($line: expr) => { $line.contains("COMMENT") };
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
pub struct Rinex {
    /// `header` field contains general information
    pub header: header::Header,
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
            header: header::Header::default(),
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

impl Rinex {
    /// Builds a new `RINEX` struct from given header & body sections
    pub fn new (header: header::Header, record: record::Record) -> Rinex {
        Rinex {
            header,
            record,
            comments: record::Comments::new(),
        }
    }

    /// Returns a copy of self but with given header attributes
    pub fn with_header (&self, header: header::Header) -> Self {
        Rinex {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
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
        let epoch : epoch::Epoch = match rtype {
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
        // Grab first 80 bytes to fully determine the BufferedReader attributes.
        // We use the `BufferedReader` wrapper for efficient file browsing (.lines())
        // and at the same time, integrated (hidden in .lines() iteration) decompression.
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
        }

/*
 *      deflate (.gzip) fd pointer does not work / is not fully supported
 *      at the moment. Let's recreate a new object, it's a little bit
 *      silly, because we actually analyze the 1st line twice,
 *      but Header builder already deduces several things from this line.
        
        reader.seek(SeekFrom::Start(0))
            .unwrap();
*/        
        let mut reader = BufferedReader::new(path)?;

        // create buffered reader
        if line.contains("CRINEX") {
            // --> enhance buffered reader
            //     with hatanaka M capacity
            reader = reader.with_hatanaka(8)?; // M = 8 is more than enough
        }

        // --> parse header fields 
        let header = header::Header::new(&mut reader)
            .unwrap();
        // --> parse record (file body)
        //     we also grab encountered comments,
        //     they might serve some fileops like `splice` / `merge` 
        let (record, comments) = record::build_record(&mut reader, &header)
            .unwrap();
        Ok(Rinex {
            header,
            record,
            comments,
        })
    }

    /// Returns true if this is an ATX RINEX 
    pub fn is_antex_rinex (&self) -> bool { self.header.rinex_type == types::Type::AntennaData }
    
    /// Returns true if this is a CLOCK RINX
    pub fn is_clocks_rinex (&self) -> bool { self.header.rinex_type == types::Type::ClockData }

    /// Returns true if this is an IONEX file
    pub fn is_ionex (&self) -> bool { self.header.rinex_type == types::Type::IonosphereMaps }

    /// Returns true if this is a METEO RINEX
    pub fn is_meteo_rinex (&self) -> bool { self.header.rinex_type == types::Type::MeteoData }
    
    /// Retruns true if this is an NAV RINX
    pub fn is_navigation_rinex (&self) -> bool { self.header.rinex_type == types::Type::NavigationData }

    /// Retruns true if this is an OBS RINX
    pub fn is_observation_rinex (&self) -> bool { self.header.rinex_type == types::Type::ObservationData }

    /// Returns `epoch` of first observation
    pub fn first_epoch (&self) -> Option<epoch::Epoch> {
        let epochs = self.epochs();
        if epochs.len() == 0 {
            None
        } else {
            Some(epochs[0])
        }
    }

    /// Returns `epoch` of last observation
    pub fn last_epoch (&self) -> Option<epoch::Epoch> {
        let epochs = self.epochs();
        if epochs.len() == 0 {
            None
        } else {
            Some(epochs[epochs.len()-1])
        }
    }

    /// Returns a list of epochs that present a data gap.
    /// Data gap is determined by comparing |e(k)-e(k-1)|: successive epoch intervals,
    /// to the INTERVAL field found in the header.
    /// Granularity is currently limited to 1 second. 
    /// This method will not produce anything if header does not an INTERVAL field.
    pub fn data_gap (&self) -> Vec<epoch::Epoch> {
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
    pub fn epoch_anomalies (&self, mask: Option<epoch::EpochFlag>) -> Vec<epoch::Epoch> { 
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
    pub fn event_description (&self, event: epoch::Epoch) -> Option<&str> {
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

    /// Splits merged `records` into seperate `records`.
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

    /// Splits record into two at desired `epoch`.
    /// Self does not have to be a `Merged` file.
    pub fn split_record_at_epoch (&self, epoch: epoch::Epoch) -> Result<(record::Record,record::Record), SplitError> {
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
    pub fn epochs (&self) -> Vec<epoch::Epoch> {
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
    
    /// Filters out epochs in record that do not have
    /// an Epoch::Ok flag attached to them.
    /// Has no effect on non Observation Records.
    pub fn epoch_ok_filter_mut (&mut self) {
        if !self.is_observation_rinex() {
            return ; // nothing we can do
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record.retain(|e, _| e.flag.is_ok());
    }

    /// Filters out epochs marker with an Ok flag.
    /// This has no effect on non observation data.
    pub fn epoch_nok_filter_mut (&mut self) {
        if !self.is_observation_rinex() {
            return ; // nothing we can do
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record.retain(|e, _| !e.flag.is_ok())
    }
    
    /// see [epoch_ok_filter_mut]
    pub fn epoch_ok_filter (&self) -> Self {
        if !self.is_observation_rinex() {
            return self.clone() // nothing we can do
        }
        let header = self.header.clone();
        let mut record = self.record.as_obs()
            .unwrap()
            .clone();
        record.retain(|e,_| e.flag.is_ok());
        Self {
            header,
            comments: self.comments.clone(),
            record: record::Record::ObsRecord(record.clone()),
        }
    }
    
    /// see [epoch_nok_filter_mut]
    pub fn epoch_nok_filter (&self) -> Self {
        if !self.is_observation_rinex() {
            return self.clone() // nothing we can do
        }
        let header = self.header.clone();
        let mut record = self.record.as_obs()
            .unwrap()
            .clone();
        record.retain(|e,_| !e.flag.is_ok());
        Self {
            header,
            comments: self.comments.clone(),
            record: record::Record::ObsRecord(record.clone()),
        }
    }
    
    /// Returns epochs where a loss of lock event happened.
    /// Has no effects on non Observation Records.
    pub fn lock_loss_events (&self) -> Vec<epoch::Epoch> {
        self
            .lli_filter(observation::record::LliFlags::LOCK_LOSS)
            .epochs()
    }

    /// Removes in place, epochs where Lock was lost
    pub fn lock_loss_filter (&mut self) {
        self
            .lli_filter_mut(observation::record::LliFlags::LOCK_LOSS)
    }

    /// Retains data that were recorded only for given list of 
    /// space vehicules. This has no effect on ATX, CLK, MET
    /// and IONEX records. 
    pub fn space_vehicule_filter_mut (&mut self, filter: Vec<sv::Sv>) {
        if self.is_observation_rinex() {
            let record = self.record
                .as_mut_obs()
                .unwrap();
            for (_e, (_clk, sv)) in record.iter_mut() {
                sv.retain(|sv, _| filter.contains(sv))
            }
        } else if self.is_navigation_rinex() {

        }
    }

    /// Computes average epoch duration
    pub fn average_epoch_duration (&self) -> std::time::Duration {
        let mut sum = 0;
        let epochs = self.epochs();
        for i in 1..epochs.len() {
            sum += (epochs[i].date - epochs[i-1].date).num_seconds() as u64
        }
        std::time::Duration::from_secs(sum / epochs.len() as u64)
    }

    /// Returns list of observables, in the form 
    /// of standardized 3 letter codes, that can be found in this record.
    /// This does not produce anything in case of ATX records.
    pub fn list_observables (&self) -> Vec<String> {
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
            for (_, sv) in record {
                for (sv, data) in sv {
                    for (k, _) in data {
                        result.push(format!("{}:{}", 
                            sv.constellation.to_3_letter_code(),
                            k.to_string()))
                    }
                }
            }
        } else if self.is_ionex() {

        }
        result
    }
/*
    /// Cleans up, in place, epochs where `loss of lock` events happened.
    /// Has no effect on non Observation Records.
    pub fn lock_loss_cleanup (&mut self) {
        if !self.is_observation_rinex() {
            return; // nothing we can do
        }
        let record = self.record
            .as_mut_obs()
            .unwrap();
        record.retain(|e, (clk, sv)| {
            sv.iter()
              .map(|(_vehicule, obs)| {
                obs.iter()
                   .map(|(_, data)| {
                      if let Some(lli) = data.lli {
                        lli == observation::record::lli_flags::LOCK_LOSS
                      } else {
                        true // lli not provided --> nothing we can do
                      }
                   })
              })
        });
    }
*/
    /// Executes in place given LLI AND mask filter.
    /// This method is very useful to determine where
    /// loss of lock or external events happened and their nature.
    /// This has no effect on non observation records.
    /// Data that do not have an LLI attached to them get also dropped out.
    pub fn lli_filter_mut (&mut self, mask: observation::record::LliFlags) {
        if !self.is_observation_rinex() {
            return ; // nothing we can do
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

    /// See [lli_filter_mut]
    pub fn lli_filter (&self, mask: observation::record::LliFlags) -> Self {
        if !self.is_observation_rinex() {
            return self.clone(); // nothing we can do
        }
        let mut record = self.record
            .as_obs()
            .unwrap()
            .clone();
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
        Self {
            record: record::Record::ObsRecord(record),
            comments: self.comments.clone(),
            header: self.header.clone(),
        }
    }

    /// Decimates record to fit minimum required epoch interval.
    /// All epochs that do not match the requirement
    /// |e(k).date - e(k-1).date| < interval, get thrown away.
    /// Also note we adjust the INTERVAL field,
    /// meaning, further file production will be correct.
    pub fn decimate_by_interval_mut (&mut self, interval: std::time::Duration) {
        let min_requirement = chrono::Duration::from_std(interval)
            .unwrap()
            .num_seconds();
        let mut last_preserved = self.epochs()[0].date;
        match self.header.rinex_type {
            types::Type::NavigationData => {
                let record = self.record
                    .as_mut_nav()
                    .unwrap();
                record.retain(|e, _| {
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
            _ => todo!("implement other record types")
        }
    }

    /// Refer to [decimate_by_interval], non mutable implementation
    pub fn decimate_by_interval(&self, interval: std::time::Duration) -> Self {
        let min_requirement = chrono::Duration::from_std(interval)
            .unwrap()
            .num_seconds();
        let mut last_preserved = self.epochs()[0].date;
        let record: record::Record = match self.header.rinex_type {
            types::Type::NavigationData => {
                let mut record = self.record
                    .as_nav()
                    .unwrap()
                    .clone();
                record.retain(|e, _| {
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
                    let delta = (e.date - last_preserved).num_seconds();
                    if e.date != last_preserved { // trick to avoid 1st entry..
                        if delta >= min_requirement {
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
            _ => todo!("implement other record types"),
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

    /// Writes self into given file.   
    /// Both header + record will strictly follow RINEX standards.   
    /// Record: supports all known `RINEX` types
    pub fn to_file (&self, path: &str) -> std::io::Result<()> {
        let mut writer = std::fs::File::create(path)?;
        write!(writer, "{}", self.header.to_string())?;
        self.record.to_file(&self.header, writer)
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
