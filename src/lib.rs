//! This package provides a set of tools to parse 
//! `RINEX` files.
//! 
//! Refer to README for example of use.  
//! Homepage: <https://github.com/gwbres/rinex>
mod leap;
mod meteo;
mod clocks;
//mod gnss_time;
mod navigation;
mod observation;

pub mod sv;
pub mod types;
pub mod epoch;
pub mod header;
pub mod record;
pub mod version;
pub mod hatanaka;
pub mod constellation;

use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use itertools::Itertools;
use std::collections::HashMap;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_comment {
    ($line: expr) => { $line.contains("COMMENT") };
}

/// `Rinex` describes a `RINEX` file
#[derive(Debug)]
pub struct Rinex {
    /// `header` field contains general information
    pub header: header::Header,
    /// `comments` : list of extra readable information,   
    /// found in `record` section exclusively.    
    /// Comments extracted from `header` sections are exposed in `header.comments`
    comments: record::Comments, 
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
}

#[derive(Error, Debug)]
/// `RINEX` merge ops related errors
pub enum MergeError {
    #[error("file types mismatch: cannot merge different `rinex`")]
    FileTypeMismatch,
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

    /// Builds a `RINEX` from given file.
    /// Header section must respect labelization standards,   
    /// some are mandatory.   
    /// Parses record for supported `RINEX` types
    pub fn from_file (path: &str) -> Result<Rinex, Error> {
        let header = header::Header::new(path)?;
        let (record, comments) = record::build_record(path, &header)?;
        Ok(Rinex {
            header,
            record,
            comments,
        })
    }

    /// Retruns true if this is an NAV rinex
    pub fn is_navigation_rinex (&self) -> bool { self.header.rinex_type == types::Type::NavigationMessage }
    /// Retruns true if this is an OBS rinex
    pub fn is_observation_rinex (&self) -> bool { self.header.rinex_type == types::Type::ObservationData }
    /// Returns true if this is a METEO rinex
    pub fn is_meteo_rinex (&self) -> bool { self.header.rinex_type == types::Type::MeteoData }

    /// Returns `epoch` (sampling timestamp) of first observation
    pub fn first_epoch (&self) -> Option<epoch::Epoch> {
        let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
            types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
            types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
            types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
        };
        if epochs.len() == 0 {
            None
        } else {
            Some(*epochs[0])
        }
    }

    /// Returns `epoch` (sampling timestamp) of last observation
    pub fn last_epoch (&self) -> Option<epoch::Epoch> {
        let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
            types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
            types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
            types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
        };
        if epochs.len() == 0 {
            None
        } else {
            Some(*epochs[epochs.len()-1])
        }
    }

    /// Returns sampling interval for rinex record 
    /// + either directly from optionnal information contained in `header`   
    /// + or (if not provided by header), by computing the average time interval between two successive epochs,    
    ///   in the extracted `record`. Only valid epochs (EpochFlag::Ok) contribute to the calculation in this case.   
    ///   Returns None, in this case if record contains a unique epoch and calculation is not feasible.
    pub fn sampling_interval (&self) -> Option<std::time::Duration> {
        if let Some(interval) = self.header.sampling_interval {
            Some(std::time::Duration::from_secs(interval as u64))
        } else {
            // build epoch interval histogram 
            let mut histogram : HashMap<i64, u64> = HashMap::new(); // {internval, population}
            let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
                types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
                types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
                types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
            };
            for i in 0..epochs.len()-1 {
                let e_i = epochs.get(i).unwrap();
                if e_i.flag.is_ok() {
                    if let Some(e) = epochs.get(i+1) {
                        if e.flag.is_ok() {
                            // delta(i+1, i) --> positive deltas
                            let delta = (e.date - epochs.get(i).unwrap().date).num_seconds();
                            if histogram.contains_key(&delta) {
                                let prev = histogram.get(&delta).unwrap();
                                histogram.insert(delta, *prev +1); // overwrite 
                            } else {
                                histogram.insert(delta, 1); // new entry
                            }
                        }
                    }
                }
            }
            let mut sorted = histogram
                .iter()
                .sorted_by(|a,b| b.cmp(a));
            //println!("Histogram sorted by Population: {:#?}", sorted); 
            if let Some(largest) = sorted.nth(0) { // largest population found
                Some(std::time::Duration::from_secs(*largest.0 as u64))
            } else { // histogram empty -> weird case(s)
                // record is either empty
                // or contained a unique epoch
                // --> calculation was not feasible
                None 
            }
        }
    }

    /// Returns a list of epochs that represent an unusual data gap (dead time: time without data in the record).   
    /// This is determined by computing time difference between successive epochs and comparing this value   
    /// to nominal time difference (`sampling interval`).    
    /// Only epochs validated by an `Ok` flag are taken into account in these calculations.    
    /// Granularity is 1 second.
    pub fn sampling_dead_time (&self) -> Vec<epoch::Epoch> {
        let sampling_interval = self.sampling_interval();
        let mut result : Vec<epoch::Epoch> = Vec::new();
        let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
            types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
            types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
            types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
        };
        if let Some(interval) = sampling_interval { // got a value to compare to
            for i in 0..epochs.len()-1 {
                let e_i = epochs.get(i).unwrap();
                if e_i.flag.is_ok() {
                    if let Some(e) = epochs.get(i+1) {
                        if e.flag.is_ok() {
                            let delta = (e.date - e_i.date).num_seconds() as u64;
                            if delta > interval.as_secs() {
                                result.push(**e)
                            }
                        }
                    }
                }
            }
        }
        result
    }
    
    /// Returns list of epochs where unusual events happen,    
    /// ie., epochs with an != Ok flag attached to them.    
    /// Use `mask` to provide a specific epoch events filter.    
    /// This method is very useful to determine when, in a given `record`,    
    /// special events happened and their nature, like:    
    ///   + locate power cycle failures in the record    
    ///   + determine where the receiver was physically moved in the `record` with EpochFlag::NewSiteOccupation mask filter    
    ///   + locate external events in the record
    ///   + determine special indications contained in header information with EpochFlag::HeaderInformation mask filter    
    ///   + much more
    pub fn epoch_anomalies (&self, mask: Option<epoch::EpochFlag>) -> Vec<&epoch::Epoch> { 
        let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
            types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
            types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
            types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
        };
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
            .filter(|(k,v)| *k == &event)
            .map(|(k,v)| v)
            .flatten()
            .collect();
        if comments.len() > 0 {
            Some(comments[0]) // TODO grab all content! by serializing into a single string
        } else {
            None
        }
    } 

    /// Returns `true` if self is a `merged` RINEX file,   
    /// that means results from two or more separate RINEX files merged toghether.   
    /// This is determined by the presence of a custom yet somewhat standardized `FILE MERGE` comments
    pub fn is_merged_rinex (&self) -> bool {
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
    /// Epochs are determined either by the pseudo standard `FILE MERGE` comment description,
    /// or by comment epochs inside the record
    pub fn merge_boundaries (&self) -> Vec<chrono::NaiveDateTime> {
        let epochs : Vec<&epoch::Epoch> = match self.header.rinex_type {
            types::Type::ObservationData => self.record.as_obs().unwrap().keys().collect(),
            types::Type::NavigationMessage => self.record.as_nav().unwrap().keys().collect(),
            types::Type::MeteoData => self.record.as_meteo().unwrap().keys().collect(),
        };
        self.comments
            .iter()
            .filter(|(k,v)| {
                v[0].contains("FILE MERGE") // TODO: wrapp all lines into a single one for testing purposes
            })
            .map(|(k,v)| k.date) // retain only timestamps
            .collect()
    }    

    /// Splits merged RINEX `records` into list of records 
    pub fn split (&self) -> Vec<record::Record> {
        let mut result : Vec<record::Record> = Vec::with_capacity(2);
        result
    }

    /// Merges given RINEX into self, in teqc similar fashion.   
    /// Header section are combined.    
    /// Records are appended together at desired epoch boundary
    pub fn merge (&mut self, rinex: &Self) -> Result<(), MergeError> {
        if self.header.rinex_type != rinex.header.rinex_type {
            return Err(MergeError::FileTypeMismatch)
        }
        Ok(())
    }

    /// Writes self into given file.   
    /// Both header + record will strictly follow RINEX standards.   
    /// Record: supports all known `RINEX` types
    fn to_file (&self, path: &str) -> std::io::Result<()> {
        let mut writer = std::fs::File::create(path)?;
        write!(writer, "{}", self.header.to_string())?;
        self.record.to_file(&self.header, writer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::process::Command;
    /// Runs `diff` to determines whether f1 & f2 
    /// are strictly identical or not
    fn diff_is_strictly_identical (f1: &str, f2: &str) -> Result<bool, std::string::FromUtf8Error> {
        let output = Command::new("diff")
            .arg("-q")
            .arg("-Z")
            .arg(f1)
            .arg(f2)
            .output()
            .expect("failed to execute \"diff\"");
        let output = String::from_utf8(output.stdout)?;
        Ok(output.len()==0)
    }
    #[test]
    /// Tests `Rinex` constructor against all known test resources
    fn test_lib() {
        let data_dir = env!("CARGO_MANIFEST_DIR").to_owned() + "/data";
        let test_data = vec![
			"NAV",
			//"OBS",
			//"CRNX",
			//"MET",
		];
        for data in test_data {
            let data_path = std::path::PathBuf::from(
                data_dir.to_owned() +"/" + data
            );
            for revision in std::fs::read_dir(data_path)
                .unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let rev_fullpath = &rev_path.to_str().unwrap(); 
                for entry in std::fs::read_dir(rev_fullpath)
                    .unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".");
                    let is_test_file = !entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with("-copy");
                    if !is_hidden && is_test_file {
                        // PARSER
                        println!("Parsing file: \"{}\"", full_path);
                        let rinex = Rinex::from_file(full_path);
                        assert_eq!(rinex.is_err(), false); // 1st basic test
                        // HEADER
                        let rinex = rinex.unwrap();
                        println!("{:#?}", rinex.header);
                        // RECORD
                        match data {
                            "NAV" => {
                                // NAV files checks
                                assert_eq!(rinex.header.crinex.is_none(), true);
                                assert_eq!(rinex.is_navigation_rinex(), true);
                                assert_eq!(rinex.header.obs_codes.is_none(), true);
                                assert_eq!(rinex.header.met_codes.is_none(), true);
                                let record = rinex.record.as_nav().unwrap();
                                println!("----- EPOCHs ----- \n{:#?}", record.keys());
                                let mut epochs = record.keys();
                                // Testing event description finder
                                if let Some(event) = epochs.nth(0) {
                                    // [!] with dummy t0 = 1st epoch timestamp
                                    //     this will actually return `header section` timestamps
                                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                                }
                            },
                            "OBS" => {
                                // OBS files checks
                                assert_eq!(rinex.header.crinex.is_none(), true);
                                assert_eq!(rinex.is_observation_rinex(), true);
                                assert_eq!(rinex.header.obs_codes.is_some(), true);
                                assert_eq!(rinex.header.met_codes.is_none(), true);
                                if rinex.header.rcvr_clock_offset_applied {
                                    // epochs should always have a RCVR clock offset
                                    // test that with iterator
                                }
                                let record = rinex.record.as_obs().unwrap();
                                let mut epochs = record.keys();
                                println!("----- EPOCHs ----- \n{:#?}", record.keys());
                                // Testing event description finder
                                if let Some(event) = epochs.nth(0) {
                                    // [!] with dummy t0 = 1st epoch timestamp
                                    //     this will actually return `header section` timestamps
                                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                                }
                            },
                            "CRNX" => {
                                // compressed OBS files checks
                                assert_eq!(rinex.header.crinex.is_some(), true);
                                assert_eq!(rinex.is_observation_rinex(), true);
                                assert_eq!(rinex.header.obs_codes.is_some(), true);
                                assert_eq!(rinex.header.met_codes.is_none(), true);
                                let record = rinex.record.as_obs().unwrap();
                                let mut epochs = record.keys();
                                println!("----- EPOCHs ----- \n{:#?}", epochs); 
                                // Testing event description finder
                                if let Some(event) = epochs.nth(0) {
                                    // [!] with dummy t0 = 1st epoch timestamp
                                    //     this will actually return `header section` timestamps
                                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                                }
                            },
							"MET" => {
                                // METEO files checks
                                assert_eq!(rinex.header.crinex.is_none(), true);
                                assert_eq!(rinex.is_meteo_rinex(), true);
                                assert_eq!(rinex.header.met_codes.is_some(), true);
                                assert_eq!(rinex.header.obs_codes.is_none(), true);
                                let record = rinex.record.as_meteo().unwrap();
                                let mut epochs = record.keys();
                                println!("----- EPOCHs ----- \n{:#?}", epochs);
                                // Testing event description finder
                                if let Some(event) = epochs.nth(0) {
                                    // [!] with dummy t0 = 1st epoch timestamp
                                    //     this will actually return `header section` timestamps
                                    println!("EVENT @ {:#?} - description: {:#?}", event, rinex.event_description(*event)); 
                                }
                            },
                            _ => {}
                        }
                        // SPECIAL METHODS
                        println!("sampling interval  : {:#?}", rinex.sampling_interval());
                        println!("sampling dead time : {:#?}", rinex.sampling_dead_time());
                        println!("abnormal epochs    : {:#?}", rinex.epoch_anomalies(None));
                        // COMMENTS
                        println!("---------- Header Comments ----- \n{:#?}", rinex.header.comments);
                        println!("---------- Body   Comments ------- \n{:#?}", rinex.comments);
                        // MERGED RINEX special ops
                        println!("---------- Merged RINEX special ops -----------\n");
                        println!("is merged          : {}", rinex.is_merged_rinex());
                        println!("boundaries: \n{:#?}", rinex.merge_boundaries());
                        // RINEX Production
                        rinex.to_file(&format!("{}-copy", full_path)).unwrap();
                        //TODO test bench
                        //let identical = diff_is_strictly_identical("test", "data/MET/V2/abvi0010.15m").unwrap();
                        //assert_eq!(identical, true)
                    }
                }
            }
        }
    }
}
