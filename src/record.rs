//! record.rs describes `RINEX` file content
use thiserror::Error;
use std::fs::File;
use std::str::FromStr;
use std::io::{self, prelude::*, BufReader};
use itertools::Itertools;
use std::collections::{BTreeMap, HashMap};

use crate::sv;
use crate::epoch;
use crate::meteo;
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
}

/// Comments: alias to describe comments encountered in `record` file section
pub type Comments = BTreeMap<epoch::Epoch, Vec<String>>;

impl Record {
	/// Returns Navigation `record`
    pub fn as_nav (&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(e) => Some(e),
            _ => None,
        }
    }
	/// Returns Observation `record`
    pub fn as_obs (&self) -> Option<&observation::Record> {
        match self {
            Record::ObsRecord(e) => Some(e),
            _ => None,
        }
    }
	/// Returns Meteo Observation `record`
    pub fn as_meteo (&self) -> Option<&meteo::Record> {
        match self {
            Record::MeteoRecord(e) => Some(e),
            _ => None,
        }
    }
    /// streams self into given file writer
    pub fn to_file (&self, header: &header::Header, mut writer: std::fs::File) -> std::io::Result<()> {
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
            Type::NavigationMessage => {
                let record = self.as_nav()
                    .unwrap();
                Ok(navigation::to_file(header, &record, writer)?)
            },
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
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
    if is_comment!(line) {
        return false
    }
	match header.version.major {
		1|2|3 => {
			// old RINEX
			// epoch block is type dependent
			match &header.rinex_type {
				Type::NavigationMessage => {
					// old NAV: epoch block
					//  is constellation dependent
					match &header.constellation {
						Some(Constellation::Glonass) => { // GLONASS NAV special case
							//  constellation ID is implied
							parsed.len() > 4
						},
						Some(_) => { // other constellations
                    		let known_sv_identifiers: &'static [char] = 
                        		&['R','G','E','B','J','C','S']; 
                            match line.chars().nth(0) {
                                Some(c) => {
									// epochs start with a known 
									//  constellation identifier
									known_sv_identifiers.contains(&c)
								},
                                _ => false
                            }
						},
                        _ => panic!("undefined constellation system")
					}
				},
				Type::ObservationData | Type::MeteoData => {
					match header.version.major {
						1|2 => {
							if parsed.len() > 6 {
								//  * contains at least 6 items
								let mut datestr = parsed[0].to_owned(); // Y
								datestr.push_str(" ");
								datestr.push_str(parsed[1]); // m
								datestr.push_str(" ");
								datestr.push_str(parsed[2]); // d
								datestr.push_str(" ");
								datestr.push_str(parsed[3]); // h
								datestr.push_str(" ");
								datestr.push_str(parsed[4]); // m
								datestr.push_str(" ");
								datestr.push_str(parsed[5]); // s
								//  * and items[0..5] do match an epoch descriptor
								epoch::str2date(&datestr).is_ok()
							} else {
								false // does not match
									// an epoch descriptor
							}
						},
						_ => {
							// OBS::V3 behaves like all::V4
							match line.chars().nth(0) {
								Some(c) => {
									c == '>' // epochs always delimited
										// by this new identifier
								},
								_ => false,
							}
						},
					}
				},
			}
		},
		_ => {
			// modern V > 3 RINEX
			// mostly easy, but Meteo seems to still follow previous format
			match &header.rinex_type {
				Type::MeteoData => {
					panic!("meteo + V4 is not fully supported yet")
				},
				_ => {
					// modern, easy parsing,
					// similar to OBS V3
            		match line.chars().nth(0) {
                		Some(c) => {
							c == '>' // epochs always delimited 
								// by this new identifier
						},
                		_ => false,
					}
				}
			}
		}
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
    let crx_info = header.crinex.as_ref();
    let mut decompressor = hatanaka::Decompressor::new(8);
    // record 
    let mut nav_rec : navigation::Record = BTreeMap::new();  // NAV
    let mut obs_rec : observation::Record = BTreeMap::new(); // OBS
    let mut met_rec : meteo::Record = BTreeMap::new();       // MET

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
        content = match crx_info {
            None => Some(line.to_string()), 
            Some(_) => {
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
                        Type::NavigationMessage => {
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
                                comment_ts = e.clone(); // for comments classification + management
                            }
                        },
                        Type::ObservationData => {
                            if let Ok((e, ck_offset, map)) = observation::build_record_entry(&header, &epoch_content) {
                                // <o 
                                // OBS data provides all observations realized @ a given epoch
                                // we should never face parsed epoch that were previously parsed
                                // even in case of `merged` RINEX
                                obs_rec.insert(e, (ck_offset, map));
                                comment_ts = e.clone(); // for comments classification + management
                            }
                        },
                        Type::MeteoData => {
                            if let Ok((e, map)) = meteo::build_record_entry(&header, &epoch_content) {
                                // <o 
                                // OBS data provides all observations realized @ a given epoch
                                // we should never face parsed epoch that were previously parsed
                                // even in case of `merged` RINEX
                                met_rec.insert(e, map);
                                comment_ts = e.clone(); // for comments classification + management
                            }
                        },
                    }

                    // new comments ?
                    if !comment_content.is_empty() {
                        comments.insert(comment_ts, comment_content.clone());
                        comment_content.clear() // reset 
                    }
                }

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
        Type::NavigationMessage => {
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
    }
    // new comments ?
    if !comment_content.is_empty() {
        comments.insert(comment_ts, comment_content.clone());
    }
    // wrap record
    let record = match &header.rinex_type {
        Type::NavigationMessage => Record::NavRecord(nav_rec),
        Type::ObservationData => Record::ObsRecord(obs_rec), 
		Type::MeteoData => Record::MeteoRecord(met_rec),
    };
    Ok((record, comments))
}
