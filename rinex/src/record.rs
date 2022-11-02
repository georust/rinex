//! `RINEX` file content description and parsing
use thiserror::Error;
use std::io::prelude::*;
use std::collections::{BTreeMap, HashMap};

use crate::antex;
use crate::epoch;
use crate::meteo;
use crate::clocks;
use crate::header;
use crate::navigation;
use crate::observation;
use crate::ionosphere;
use crate::is_comment;
use crate::types::Type;
use crate::reader::BufferedReader;
use crate::writer::BufferedWriter;
use crate::hatanaka::{
    Compressor, Decompressor,
};

use crate::merge;
use merge::Merge;

use crate::split;
use split::Split;

#[cfg(feature = "serde")]
use serde::Serialize;

/// `Record`
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Record {
    /// ATX record, see [antex::record::Record] 
    AntexRecord(antex::Record),
    /// Clock record, see [clocks::record::Record] 
    ClockRecord(clocks::Record),
	/// IONEX (ionosphere maps) record
    IonexRecord(ionosphere::Record),
	/// Meteo record, see [meteo::record::Record]
    MeteoRecord(meteo::Record),
	/// Navigation record, see [navigation::record::Record]
    NavRecord(navigation::Record),
	/// Observation record, see [observation::record::Record]
    ObsRecord(observation::Record),
}

/// Record comments are high level informations, sorted by epoch
/// (timestamp) of appearance. We deduce the "associated" timestamp from the
/// previosuly parsed epoch, when parsing the record.
pub type Comments = BTreeMap<epoch::Epoch, Vec<String>>;

impl Record {
    /// Unwraps self as ANTEX record
    pub fn as_antex (&self) -> Option<&antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable reference to ANTEX record
    pub fn as_mut_antex (&mut self) -> Option<&mut antex::Record> {
        match self {
            Record::AntexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as CLK record
    pub fn as_clock (&self) -> Option<&clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable CLK record
    pub fn as_mut_clock (&mut self) -> Option<&mut clocks::Record> {
        match self {
            Record::ClockRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as IONEX record
    pub fn as_ionex (&self) -> Option<&ionosphere::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps self as mutable IONEX record
    pub fn as_mut_ionex (&mut self) -> Option<&mut ionosphere::Record> {
        match self {
            Record::IonexRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Unwraps self as MET record
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
	/// Unwraps self as NAV record
    pub fn as_nav (&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Returns mutable reference to Navigation record
    pub fn as_mut_nav (&mut self) -> Option<&mut navigation::Record> {
        match self {
            Record::NavRecord(r) => Some(r),
            _ => None,
        }
    }
	/// Unwraps self as OBS record
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
    pub fn to_file (&self, header: &header::Header, writer: &mut BufferedWriter) -> Result<(), Error> {
        match &header.rinex_type {
            Type::MeteoData => {
                let record = self.as_meteo()
                    .unwrap();
                for (epoch, data) in record.iter() {
                    if let Ok(epoch) = meteo::fmt_epoch(epoch, data, header) {
                        let _ = write!(writer, "{}", epoch);
                    }
                }
            },
            Type::ObservationData => {
                let record = self.as_obs()
                    .unwrap();
                let is_crinex = header.is_crinex();
                let mut compressor = Compressor::new();
                for (epoch, (clock_offset, data)) in record.iter() {
                    let epoch = observation::fmt_epoch(epoch, clock_offset, data, header);
                    if is_crinex {
                        for line in epoch.lines() {
                            let line = line.to_owned() + "\n"; // helps the following .lines() iterator
                                // embedded in compression method
                            if let Ok(compressed) = compressor.compress(&header, &line) {
                                write!(writer, "{}", compressed)?;
                            }
                        }
                    } else {
                        write!(writer, "{}", epoch)?;
                    }
                }
            },
            Type::NavigationData => {
                let record = self.as_nav()
                    .unwrap();
                for (epoch, classes) in record.iter() {
                    if let Ok(epoch) = navigation::fmt_epoch(epoch, classes, header) {
                        let _ = write!(writer, "{}", epoch);
                    }
                }
            },
			Type::ClockData => {
				let record = self.as_clock()
					.unwrap();
				for (epoch, data) in record.iter() {
                    if let Ok(epoch) = clocks::fmt_epoch(epoch, data) {
                        let _ = write!(writer, "{}", epoch); 
				    }
                }
			},
            _ => panic!("record type not supported yet"),
        }
        Ok(())
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
    FileIoError(#[from] std::io::Error),
    #[error("failed to produce Navigation epoch")]
    NavEpochError(#[from] navigation::Error),
    #[error("failed to produce Clock epoch")]
    ClockEpochError(#[from] clocks::Error),
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
        Type::IonosphereMaps => ionosphere::is_new_map(line),
        Type::NavigationData => navigation::is_new_epoch(line, header.version), 
        Type::ObservationData => observation::is_new_epoch(line, header.version),
        Type::MeteoData => meteo::is_new_epoch(line, header.version),
    }
}

/// Builds a `Record`, `RINEX` file body content,
/// which is constellation and `RINEX` file type dependent
pub fn parse_record (reader: &mut BufferedReader, header: &header::Header) -> Result<(Record, Comments), Error> {
    let mut first_epoch = true;
    let mut content : Option<String>; // epoch content to build
    let mut epoch_content = String::with_capacity(6*64);
    let mut exponent: i8 = -1; //IONEX record scaling: this is the default value
    
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
    let mut decompressor = Decompressor::new();
    // record 
    let mut atx_rec = antex::Record::new(); // ATX
    let mut nav_rec = navigation::Record::new(); // NAV
    let mut obs_rec = observation::Record::new(); // OBS
    let mut met_rec = meteo::Record::new(); // MET
    let mut clk_rec = clocks::Record::new(); // CLK
    let mut ionx_rec = ionosphere::Record::new(); //IONEX

    for l in reader.lines() { // iterates one line at a time 
        let line = l.unwrap();
        // COMMENTS special case
        // --> store
        // ---> append later with epoch.timestamp attached to it
        if is_comment!(line) {
            let comment = line.split_at(60).0.trim_end();
            comment_content.push(comment.to_string());
            continue
        }
        // IONEX exponent-->data scaling
        // hidden to user and allows high level interactions
        if line.contains("EXPONENT") {
            let content = line.split_at(60).0;
            if let Ok(e) = i8::from_str_radix(content.trim(), 10) {
                exponent = e // --> update current exponent value
            }
        }
        // manage CRINEX case
        //  [1]  RINEX : pass content as is
        //  [2] CRINEX : decompress
        //           --> decompressed content will probably wind up as more than one line
        content = match crinex {
            false => { // RINEX context
				if line.len() == 0 {
					Some(String::from("\n")) // helps all following .lines() iteration,
						// contained in xx_parse_epoch()
				} else {
					Some(line.to_string())
				}
			},
            true => { // CRINEX context
                // append an \n to help the .line browser contained in .decompress()
                //  we might use a different browsing method in the future
                //  like decompress_line() which expects a complete line, whatever happens
                let mut l = line.to_owned();
                l.push_str("\n");
                if let Ok(recovered) = decompressor.decompress(&header, &l) {
                    Some(recovered)
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
                            let constellation = &header.constellation.unwrap();
                            if let Ok((e, class, fr)) = navigation::parse_epoch(header.version, *constellation, &epoch_content) {
                                if let Some(e) = nav_rec.get_mut(&e) {
                                    // epoch already encountered
                                    if let Some(frames) = e.get_mut(&class) {
                                        // class already encountered for this epoch
                                        frames.push(fr);
                                    } else {
                                        // new class entry, for this epoch
                                        let mut inner: Vec<navigation::Frame> = Vec::with_capacity(1);
                                        inner.push(fr);
                                        e.insert(class, inner);
                                    }
                                } else { // new epoch: create entry entry
                                    let mut map: BTreeMap<navigation::FrameClass, Vec<navigation::Frame>> = BTreeMap::new();
                                    let mut inner: Vec< navigation::Frame> = Vec::with_capacity(1);
                                    inner.push(fr);
                                    map.insert(class, inner);
                                    nav_rec.insert(e, map);
                                }
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::ObservationData => {
                            if let Ok((e, ck_offset, map)) = observation::parse_epoch(&header, &epoch_content) {
                                obs_rec.insert(e, (ck_offset, map));
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::MeteoData => {
                            if let Ok((e, map)) = meteo::parse_epoch(&header, &epoch_content) {
                                met_rec.insert(e, map);
                                comment_ts = e.clone(); // for comments classification & management
                            }
                        },
                        Type::ClockData => {
                            if let Ok((epoch, dtype, system, data)) = clocks::parse_epoch(header.version, &epoch_content) {
                                if let Some(e) = clk_rec.get_mut(&epoch) {
                                    if let Some(d) = e.get_mut(&dtype) {
                                        d.insert(system, data);
                                    } else {
                                        // --> new system entry for this `epoch`
                                        let mut inner: HashMap<clocks::System, clocks::Data> = HashMap::new();
                                        inner.insert(system, data);
                                        e.insert(dtype, inner);
                                    }
                                } else {
                                    // --> new epoch entry
                                    let mut inner: HashMap<clocks::System, clocks::Data> = HashMap::new();
                                    inner.insert(system, data);
                                    let mut map : HashMap<clocks::DataType, HashMap<clocks::System, clocks::Data>> = HashMap::new();
                                    map.insert(dtype, inner);
                                    clk_rec.insert(epoch, map);
                                }
                                comment_ts = epoch.clone(); // for comments classification & management
                            }
                        },
                        Type::AntennaData => {
                            if let Ok((antenna, frequencies)) = antex::parse_epoch(&epoch_content) {
                                let mut found = false;
                                for (ant, freqz) in atx_rec.iter_mut() {
                                    if *ant == antenna {
                                        for f in frequencies.iter() {
                                            freqz.push(f.clone());
                                        }
                                        found = true;
                                        break
                                    }
                                }
                                if !found {
                                    atx_rec.push((antenna, frequencies));
                                }
                            }
                        },
                        Type::IonosphereMaps => {
                            if let Ok((epoch, map)) = ionosphere::parse_epoch(&epoch_content, exponent) {
                                ionx_rec.insert(epoch, (map, None, None));
                            }
                        }
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
            let constellation = &header.constellation.unwrap();
            if let Ok((e, class, fr)) = navigation::parse_epoch(header.version, *constellation, &epoch_content) {
                if let Some(e) = nav_rec.get_mut(&e) {
                    // epoch already encountered
                    if let Some(frames) = e.get_mut(&class) {
                        // class already encountered for this epoch
                        frames.push(fr);
                    } else {
                        // new class entry, for this epoch
                        let mut inner: Vec<navigation::Frame> = Vec::with_capacity(1);
                        inner.push(fr);
                        e.insert(class, inner);
                    }
                } else { // new epoch: create entry entry
                    let mut map: BTreeMap<navigation::FrameClass, Vec<navigation::Frame>> = BTreeMap::new();
                    let mut inner: Vec<navigation::Frame> = Vec::with_capacity(1);
                    inner.push(fr);
                    map.insert(class, inner);
                    nav_rec.insert(e, map);
                }
                comment_ts = e.clone(); // for comments classification & management
            }
        },
        Type::ObservationData => {
            if let Ok((e, ck_offset, map)) = observation::parse_epoch(&header, &epoch_content) {
                obs_rec.insert(e, (ck_offset, map));
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::MeteoData => {
            if let Ok((e, map)) = meteo::parse_epoch(&header, &epoch_content) {
                met_rec.insert(e, map);
                comment_ts = e.clone(); // for comments classification + management
            }
        },
        Type::ClockData => {
            if let Ok((e, dtype, system, data)) = clocks::parse_epoch(header.version, &epoch_content) {
                // Clocks `RINEX` files are handled a little different,
                // because we parse one line at a time, while we parsed one epoch at a time for other RINEXes.
                // One line may contribute to a previously existing epoch in the record 
                // (different type of measurements etc..etc..)
                if let Some(e) = clk_rec.get_mut(&e) {
                    if let Some(d) = e.get_mut(&dtype) {
                        d.insert(system, data);
                    } else {
                        // --> new system entry for this `epoch`
                        let mut map: HashMap<clocks::DataType, HashMap<clocks::System, clocks::Data>> = HashMap::new();
                        let mut inner: HashMap<clocks::System, clocks::Data> = HashMap::new();
                        inner.insert(system, data);
                        map.insert(dtype, inner);
                    }
                } else {
                    // --> new epoch entry
                    let mut map: HashMap<clocks::DataType, HashMap<clocks::System, clocks::Data>> = HashMap::new();
                    let mut inner: HashMap<clocks::System, clocks::Data> = HashMap::new();
                    inner.insert(system, data);
                    map.insert(dtype, inner);
                    clk_rec.insert(e, map);
                }
                comment_ts = e.clone(); // for comments classification & management
            }
        },
        Type::IonosphereMaps => {
            if let Ok((epoch, maps)) = ionosphere::parse_epoch(&epoch_content, exponent) {
                ionx_rec.insert(epoch, (maps, None, None));
            }
        }
        Type::AntennaData => {
            if let Ok((antenna, frequencies)) = antex::parse_epoch(&epoch_content) {
                let mut found = false;
                for (ant, freqz) in atx_rec.iter_mut() {
                    if *ant == antenna {
                        for f in frequencies.iter() {
                            freqz.push(f.clone());
                        }
                        found = true;
                        break
                    }
                }
                if !found {
                    atx_rec.push((antenna, frequencies));
                }
            }
        },
    }
    // new comments ?
    if !comment_content.is_empty() {
        comments.insert(comment_ts, comment_content.clone());
    }
    // wrap record
    let record = match &header.rinex_type {
        Type::AntennaData => Record::AntexRecord(atx_rec),
        Type::ClockData => Record::ClockRecord(clk_rec),
        Type::IonosphereMaps => Record::IonexRecord(ionx_rec),
		Type::MeteoData => Record::MeteoRecord(met_rec),
        Type::NavigationData => Record::NavRecord(nav_rec),
        Type::ObservationData => Record::ObsRecord(obs_rec), 
    };
    Ok((record, comments))
}

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge (&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        if let Some(lhs) = self.as_mut_nav() {
            if let Some(rhs) = rhs.as_nav() {
                lhs.merge_mut(&rhs)?;
            }
        }
        if let Some(lhs) = self.as_mut_obs() {
            if let Some(rhs) = rhs.as_obs() {
                lhs.merge_mut(&rhs)?;
            }
        }
        if let Some(lhs) = self.as_mut_meteo() {
            if let Some(rhs) = rhs.as_meteo() {
                lhs.merge_mut(&rhs)?;
            }
        }
        if let Some(lhs) = self.as_mut_ionex() {
            if let Some(rhs) = rhs.as_ionex() {
                lhs.merge_mut(&rhs)?;
            }
        }
        if let Some(lhs) = self.as_mut_antex() {
            if let Some(rhs) = rhs.as_antex() {
                lhs.merge_mut(&rhs)?;
            }
        }
        if let Some(lhs) = self.as_mut_clock() {
            if let Some(rhs) = rhs.as_clock() {
                lhs.merge_mut(&rhs)?;
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split_at_epoch(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        if let Some(r) = self.as_obs() {
            r.split_at_epoch(epoch)
        } else if let Some(r) = self.as_nav() {
            r.split_at_epoch(epoch)
        } else if let Some(r) = self.as_meteo() {
            r.split_at_epoch(epoch)
        } else if let Some(r) = self.as_ionex() {
            r.split_at_epoch(epoch)
        } else if let Some(r) = self.as_clock() {
            r.split_at_epoch(epoch)
        } else {
            Err(split::Error::NoEpochIteratio)
        }
    }
}
