//! `RINEX` file content description and parsing
use thiserror::Error;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::collections::{BTreeMap, HashMap};

use crate::sv;
use crate::antex;
use crate::epoch;
use crate::meteo;
use crate::clocks;
use crate::header;
use crate::hatanaka;
use crate::navigation;
use crate::observation;
use crate::is_comment;
use crate::types::Type;
use crate::constellation::Constellation;

/// `Record`
#[derive(Clone, Debug)]
pub enum Record {
	/// `navigation::Record` : Navigation Data file content.    
	/// `record` is a list of `navigation::ComplexEnum` sorted
	/// by `epoch` and by `Sv`
    NavRecord(navigation::Record),
	/// `observation::Record` : Observation Data file content.   
	/// `record` is a list of `observation::ObservationData` indexed
	/// by Observation code, sorted by `epoch` and by `Sv`
    ObsRecord(observation::Record),
	/// `meteo::Record` : Meteo Data file content.   
	/// `record` is a hashmap of f32 indexed by Observation Code,
	/// sorted by `epoch`
    MeteoRecord(meteo::Record),
    /// `clocks::Record` : CLOCKS RINEX file content
    ClockRecord(clocks::Record),
    // /// `IONEX` record is a list of Ionosphere Maps,
    // /// indexed by `epoch`
    // IonexRecord(ionex::Record),
    /// `antex::Record` : Antenna Data file content.
    /// `record` is a list of Antenna caracteristics sorted
    /// by antenna model. `ATX` records are not `epoch` iterable
    AntexRecord(antex::Record),
}

/// Comments: alias to describe comments encountered in `record` file section
pub type Comments = BTreeMap<epoch::Epoch, Vec<String>>;

impl Record {
    /// Unwraps self as ANTEX `record`
    pub fn as_antex (&self) -> Option<&antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable reference to ANTEX `record`
    pub fn as_mut_antex (&mut self) -> Option<&mut antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as CLK `record`
    pub fn as_clock (&self) -> Option<&clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable CLK `record`
    pub fn as_mut_clock (&mut self) -> Option<&mut clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
/*
    /// Unwraps self as IONEX record
    pub fn as_ionex (&self) -> Option<&ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable IONEX record
    pub fn as_mut_ionex (&mut self) -> Option<&mut ionex::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
*/
	/// Unwraps self as MET `record`
    pub fn as_meteo (&self) -> Option<&meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Meteo record
    pub fn as_mut_meteo (&mut self) -> Option<&mut meteo::Record> {
        match self {
            Record::MeteoRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Unwraps self as NAV `record`
    pub fn as_nav (&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Returns mutable reference to Navigation `record`
    pub fn as_mut_nav (&mut self) -> Option<&mut navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Unwraps self as OBS `record`
    pub fn as_obs (&self) -> Option<&observation::Record> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to Observation record
    pub fn as_mut_obs (&mut self) -> Option<&mut observation::Record> {
        match self {
            Record::ObsRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Streams into given file writer
    pub fn to_file (&self, header: &header::Header, writer: std::fs::File) -> std::io::Result<()> {
        match &header.rinex_type {
            Type::MeteoData => {
                let record = self.as_meteo()
                    .unwrap();
                Ok(meteo::to_file(header, &record, writer)?)
            },
            Type::ObservationData => {
                let record = self.as_obs()
                    .unwrap();
                Ok(observation::to_file(header, &record, writer)?)
            },
            Type::NavigationData => {
                let record = self.as_nav()
                    .unwrap();
                Ok(navigation::to_file(header, &record, writer)?)
            },
            _ => panic!("record type not supported yet"),
        }
    }
}

impl Default for Record {
    fn default() -> Record {
        Record::NavRecord(navigation::Record::new())
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("record parsing not supported for type \"{0}\"")]
    TypeError(String),
    #[error("file i/o error")]
    IoError(#[from] std::io::Error),
}

/// Returns true if given line matches the start   
/// of a new epoch, inside a RINEX record.
pub fn is_new_epoch (line: &str, header: &header::Header) -> bool {
    if is_comment!(line) {
        return false
    }
    match &header.rinex_type {
        Type::AntennaData => antex::is_new_epoch(line),
        Type::ClockData => clocks::is_new_epoch(line),
        //Type::IonosphereMaps => ionex::is_new_tec_map(line),
        Type::NavigationData => navigation::is_new_epoch(line, header.version, header.constellation.unwrap()),
        Type::ObservationData => observation::is_new_epoch(line, header.version),
        Type::MeteoData => meteo::is_new_epoch(line, header.version),
    }
}

/// Builds a `Record`, `RINEX` file body content,
/// which is constellation and `RINEX` file type dependent
pub fn build_record (path: &str, header: &header::Header) -> Result<(Record, Comments), Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut inside_header = true;
    let mut first_epoch = true;
    let mut content : Option<String>; // epoch content to build
    let mut epoch_content = String::with_capacity(6*64);
    
    // to manage `record` comments
    let mut comments : Comments = Comments::new();
    let mut comment_ts = epoch::Epoch::default();
    let mut comment_content : Vec<String> = Vec::with_capacity(4);

    // CRINEX record special process is special
    // we need the decompression algorithm to run in rolling fashion
    // and feed the decompressed result to the `new epoch` detection method
    let crinex = if let Some(obs) = &header.obs {
        obs.crinex.is_some()
    } else {
        false
    };
    let mut decompressor = hatanaka::Decompressor::new(8);
    // record 
    let mut nav_rec : navigation::Record = BTreeMap::new();  // NAV
    let mut obs_rec : observation::Record = BTreeMap::new(); // OBS
    let mut met_rec : meteo::Record = BTreeMap::new();       // MET
    let mut clk_rec : clocks::Record = BTreeMap::new();      // CLK
    let mut atx_rec : antex::Record = BTreeMap::new();       // ATX

    for l in reader.lines() { // process one line at a time 
        let line = l.unwrap();
        // HEADER : already processed
        if inside_header {
            if line.contains("END OF HEADER") {
                inside_header = false // header is ending 
            }
            continue
        }
        // COMMENTS special case
        // --> store
        // ---> append later with epoch.timestamp attached to it
        if is_comment!(line) {
            let comment = line.split_at(60).0.trim_end();
            comment_content.push(comment.to_string());
            continue
        }
        // manage CRINEX case
        //  [1]  RINEX : pass content as is
        //  [2] CRINEX : decompress
        //           --> decompressed content may wind up as more than one line
        content = match crinex {
            false => Some(line.to_string()), 
            true => {
                // decompressor::decompress()
                // splits content on \n as it can work on several lines at once,
                // here we iterate through each line, so add an extra \n
                let mut l = line.to_owned();
                l.push_str("\n");
                // --> recover compressed data
                if let Ok(recovered) = decompressor.decompress(&header, &l) {
                    let mut result = String::with_capacity(4*80);
                    for line in recovered.lines() {
                        result.push_str(line);
                        result.push_str("\n")
                    }
                    Some(result)
                } else {
                    None
                }
            },
        };

        if let Some(content) = content {
            // CRINEX decompression passed
            // or regular RINEX content passed
            // --> epoch boundaries determination
            for line in content.lines() { // may comprise several lines, in case of CRINEX
                let new_epoch = is_new_epoch(line, &header);
                if new_epoch && !first_epoch {
                    match &header.rinex_type {
                        Type::NavigationData => {
                            if let Ok((e, sv, map)) = navigation::build_record_entry(&header, &epoch_content) {
                                if nav_rec.contains_key(&e) {
                                    // <o 
                                    // NAV epoch provides a unique Sv for a given epoch
                                    // it is possible to return an already existing epoch (previously parsed)
                                    // in case of `merged` RINEX
                                    // --> retrieve previous epoch
                                    // ---> append new `sv` data 
                                    let mut prev = nav_rec.remove(&e).unwrap(); // grab previous entry
                                    prev.insert(sv, map); // insert 
                                    nav_rec.insert(e, prev); // (re)insert
                                } else {
                                    // new epoch -> insert
                                    let mut sv_map : HashMap<sv::Sv, HashMap<String, navigation::ComplexEnum>> = HashMap::with_capacity(1);
                                    sv_map.insert(sv, map);
                                    nav_rec.insert(e, sv_map);
                                };
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::ObservationData => {
                            if let Ok((e, ck_offset, map)) = observation::build_record_entry(&header, &epoch_content) {
                                // <o 
                                // OBS data provides all observations realized @ a given epoch
                                // we should never face parsed epoch that were previously parsed
                                // even in case of `merged` RINEX
                                obs_rec.insert(e, (ck_offset, map));
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::MeteoData => {
                            if let Ok((e, map)) = meteo::build_record_entry(&header, &epoch_content) {
                                // <o 
                                // OBS data provides all observations realized @ a given epoch
                                // we should never face parsed epoch that were previously parsed
                                // even in case of `merged` RINEX
                                met_rec.insert(e, map);
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::ClockData => {
                            if let Ok((epoch, system, dtype, data)) = clocks::build_record_entry(&header, &epoch_content) {
                                // Clocks `RINEX` files are handled a little different,
                                // because we parse one line at a time, while we parsed one epoch at a time for other RINEXes.
                                // One line may contribute to a previously existing epoch in the record 
                                // (different type of measurements etc..etc..)
                                if let Some(mut e) = clk_rec.get_mut(&epoch) {
                                    if let Some(mut s) = e.get_mut(&system) {
                                        s.insert(dtype, data);
                                    } else {
                                        // --> new system entry for this `epoch`
                                        let mut inner: HashMap<clocks::DataType, clocks::Data> = HashMap::new();
                                        let mut map: HashMap<clocks::System, HashMap<clocks::DataType, clocks::Data>> = HashMap::new();
                                        inner.insert(dtype, data);
                                        e.insert(system, inner);
                                    }
                                } else {
                                    // --> new epoch entry
                                    let mut inner:HashMap<clocks::DataType, clocks::Data> = HashMap::new();
                                    inner.insert(dtype, data);
                                    let mut map : HashMap<clocks::System, HashMap<clocks::DataType, clocks::Data>> = HashMap::new();
                                    map.insert(system, inner);
                                    clk_rec.insert(epoch, map);
                                }
                                comment_ts = epoch.clone(); // for comments classification & management
                            }
                        },
                        Type::AntennaData => {
                            if let Ok((antenna, frequencies)) = antex::build_record_entry(&epoch_content) {
                            /*    if let Some((_, mut v)) = atx_rec.get_mut(antenna) {
                                    v.push(frequencies) 
                                } else {
                                    atx_rec.insert(antenna, frequencies)
                                }*/
                            }
                        },
                        _ => todo!("record type not fully supported yet"),
                    }

                    // new comments ?
                    if !comment_content.is_empty() {
                        comments.insert(comment_ts, comment_content.clone());
                        comment_content.clear() // reset 
                    }
                }//is_new_epoch() +!first

                if new_epoch {
                    if !first_epoch {
                        epoch_content.clear()
                    }
                    first_epoch = false;
                }
                // epoch content builder
                epoch_content.push_str(&line);
                epoch_content.push_str("\n")
            }
        }
    }
    // --> try to build an epoch out of current residues
    // this covers 
    //   + final epoch (last epoch in record)
    //   + comments parsing with empty record (empty file body)
    match &header.rinex_type {
        Type::NavigationData => {
            if let Ok((e, sv, map)) = navigation::build_record_entry(&header, &epoch_content) {
                let mut smap : HashMap<sv::Sv, HashMap<String, navigation::ComplexEnum>> = HashMap::with_capacity(1);
                smap.insert(sv, map);
                nav_rec.insert(e, smap);
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::ObservationData => {
            if let Ok((e, ck_offset, map)) = observation::build_record_entry(&header, &epoch_content) {
                obs_rec.insert(e, (ck_offset, map));
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::MeteoData => {
            if let Ok((e, map)) = meteo::build_record_entry(&header, &epoch_content) {
                met_rec.insert(e, map);
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::ClockData => {
            if let Ok((e, system, dtype, data)) = clocks::build_record_entry(&header, &epoch_content) {
                // Clocks `RINEX` files are handled a little different,
                // because we parse one line at a time, while we parsed one epoch at a time for other RINEXes.
                // One line may contribute to a previously existing epoch in the record 
                // (different type of measurements etc..etc..)
                if let Some(mut e) = clk_rec.get_mut(&e) {
                    if let Some(mut s) = e.get_mut(&system) {
                        s.insert(dtype, data);
                    } else {
                        // --> new system entry for this `epoch`
                        let mut inner: HashMap<clocks::DataType, clocks::Data> = HashMap::new();
                        let mut map: HashMap<clocks::System, HashMap<clocks::DataType, clocks::Data>> = HashMap::new();
                        inner.insert(dtype, data);
                        map.insert(system, inner);
                    }
                } else {
                    // --> new epoch entry
                    let mut inner:HashMap<clocks::DataType, clocks::Data> = HashMap::new();
                    inner.insert(dtype, data);
                    let mut map : HashMap<clocks::System, HashMap<clocks::DataType, clocks::Data>> = HashMap::new();
                    map.insert(system, inner);
                    clk_rec.insert(e, map);
                }
                comment_ts = e.clone(); // for comments classification & management
            }
        },
        _ => todo!("record type not fully supported yet"),
    }
    // new comments ?
    if !comment_content.is_empty() {
        comments.insert(comment_ts, comment_content.clone());
    }
    // wrap record
    let record = match &header.rinex_type {
        Type::AntennaData => Record::AntexRecord(atx_rec),
        Type::NavigationData => Record::NavRecord(nav_rec),
        Type::ObservationData => Record::ObsRecord(obs_rec), 
		Type::MeteoData => Record::MeteoRecord(met_rec),
        Type::ClockData => Record::ClockRecord(clk_rec),
        _ => todo!("record type not fully supported yet"),
    };
    Ok((record, comments))
}
