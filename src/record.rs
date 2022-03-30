//! record.rs describes `RINEX` file content
use thiserror::Error;
use std::collections::HashMap;

use crate::sv;
use crate::epoch;
use crate::meteo;
use crate::header;
use crate::hatanaka;
use crate::navigation;
use crate::observation;
use crate::is_comment;
use crate::types::{Type, TypeError};
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
    // writes self into given file writer
/*    fn to_file (&self, header: &header::Header, mut writer: std::fs::File) -> Result<(), Error> {
        /*match &header.rinex_type {
            Type::MeteorologicalData => {
                let record = self.as_meteo()
                    .unwrap();
                meteo::to_file(record)?
            },
            _ => {
            },
        }*/
        Ok(())
    }*/
}

impl Default for Record {
    fn default() -> Record {
        Record::NavRecord(navigation::Record::new())
    }
}

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("record parsing not supported for type \"{0}\"")]
    TypeError(String),
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
				Type::ObservationData | Type::MeteorologicalData => {
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
				Type::MeteorologicalData => {
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
pub fn build_record (header: &header::Header, body: &str) -> Result<Record, TypeError> { 
    let mut line;
    let mut first_epoch = true;
    let mut body = body.lines();
    let mut epoch_content = String::with_capacity(6*64);

    // for CRINEX record, process is special
    // we need the decompression algorithm to run in rolling fashion
    // and feed the decompressed result to the `new epoch` detection method
    let crx_info = header.crinex.as_ref();
    let mut decompressor = hatanaka::Decompressor::new(8);
    // record 
    let mut nav_rec : navigation::Record = HashMap::new();  // NAV
    let mut obs_rec : observation::Record = HashMap::new(); // OBS
    let mut met_rec : meteo::Record = HashMap::new();       // MET
    
    loop { // iterates over each line
        if let Some(l) = body.next() {
            line = l;
        } else {
            break // EOF
        }
        
        if is_comment!(line) {
            continue  // SKIP
        }

        // manage CRINEX case
        //  [1]  RINEX : pass content as is
        //  [2] CRINEX : decompress
        //           --> decompressed content may wind up as more than one line
        let content : Option<String> = match crx_info {
            None => Some(line.to_string()), 
            Some(_) => {
                let mut l = line.to_owned();
                l.push_str("\n"); // body.next() has stripped the "\n" that recover expects
                if let Ok(recovered) = decompressor.recover(&header, &l) {
                    Some(recovered.lines().collect::<String>())
                } else {
                    None
                }
            },
        };
        
        // pack content into epoch str
        if let Some(content) = content {
            for line in content.lines() {
                let new_epoch = is_new_epoch(line, &header);
                if new_epoch && !first_epoch {
                    match &header.rinex_type {
                        Type::NavigationMessage => {
                            if let Ok((e, sv, map)) = navigation::build_record_entry(&header, &epoch_content) {
                                let mut smap : HashMap<sv::Sv, HashMap<String, navigation::ComplexEnum>> = HashMap::with_capacity(1);
                                smap.insert(sv, map);
                                nav_rec.insert(e, smap);
                            }
                        },
                        Type::ObservationData => {
                            if let Ok((e, ck_offset, map)) = observation::build_record_entry(&header, &epoch_content) {
                                /*println!("all good");
                                println!("\"{}\"", epoch_content);*/
                                obs_rec.insert(e, (ck_offset, map));
                            } /*else {
                                println!("oops");
                                println!("\"{}\"", epoch_content)
                            }*/
                        },
                        Type::MeteorologicalData => {
                            if let Ok((e, map)) = meteo::build_record_entry(&header, &epoch_content) {
                                met_rec.insert(e, map);
                            }
                        },
                    }
                }

                if new_epoch {
                    if !first_epoch {
                        epoch_content.clear()
                    }
                    first_epoch = false;
                }
                // epoch content builder
                epoch_content.push_str(&line); //.trim_end());
                epoch_content.push_str("\n")
            }
        }
    }
    match &header.rinex_type {
        Type::NavigationMessage => Ok(Record::NavRecord(nav_rec)),
        Type::ObservationData => Ok(Record::ObsRecord(obs_rec)), 
		Type::MeteorologicalData => Ok(Record::MeteoRecord(met_rec)),
    }
}
