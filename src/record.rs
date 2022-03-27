//! record.rs describes `RINEX` file content
use thiserror::Error;
use std::collections::HashMap;

use crate::epoch;
use crate::meteo;
use crate::header;
use crate::navigation;
use crate::observation;
use crate::is_comment;
use crate::{Type, TypeError};
use crate::constellation::Constellation;

/// ̀`Sv` describes a Satellite Vehiculee
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sv {
    pub prn: u8,
    pub constellation: Constellation,
}

/// ̀ Sv` related errors
#[derive(Error, Debug)]
pub enum ParseSvError {
    #[error("unknown constellation \"{0}\"")]
    UnidentifiedConstellation(char),
    #[error("failed to parse prn")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Default for Sv {
    /// Builds a default `Sv`
    fn default() -> Sv {
        Sv {
            constellation: Constellation::default(),
            prn: 0
        }
    }
}

impl Sv {
    /// Creates a new `Sv` descriptor
    pub fn new (constellation: Constellation, prn: u8) -> Sv { Sv {constellation, prn }}
}

impl std::str::FromStr for Sv {
    type Err = ParseSvError;
    /// Builds an `Sv` from string content
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        let constellation : Constellation;
        if s.starts_with('G') {
            constellation = Constellation::GPS;
        } else if s.starts_with('E') {
            constellation = Constellation::Galileo;
        } else if s.starts_with('R') {
            constellation = Constellation::Glonass;
        } else if s.starts_with('S') {
            constellation = Constellation::Sbas;
        } else if s.starts_with('J') {
            constellation = Constellation::QZSS;
        } else if s.starts_with('C') {
            constellation = Constellation::Beidou;
        } else {
            return Err(ParseSvError::UnidentifiedConstellation(s.chars().nth(0).unwrap()));
        }
        let prn = u8::from_str_radix(&s[1..].trim(), 10)?;
        Ok(Sv{constellation, prn})
    }
}

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
}

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("record parsing not supported for type \"{0}\"")]
    TypeError(String),
}

/// Returns true if given line matches the start   
/// of a new epoch, inside a RINEX record.    
/// Will panic on CRINEX data - unable to hanle it
pub fn is_new_epoch (line: &str, header: &header::RinexHeader) -> bool {
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
    if header.is_crinex() {
        panic!("is_new_epoch() for CRINEX record is not supported")
    }
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
						_ => unreachable!(), // RINEX::NAV body while Type!=NAV
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
								false  // does not match
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
			// mostly easy, but Meteo seems still follow
			// an old fashion
			match &header.rinex_type {
				Type::MeteorologicalData => {
					unreachable!()
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
pub fn build_record (header: &header::RinexHeader, body: &str) -> Result<Record, TypeError> { 
    let mut body = body.lines();
    let mut line = body.next()
        .unwrap();
    while is_comment!(line) {
        line = body.next()
            .unwrap()
    }
    let mut eof = false;
    let mut first = true;
    let mut block = String::with_capacity(256*1024); // max. block size

    let mut nav_rec : navigation::Record = HashMap::new();
    let mut obs_rec : observation::Record = HashMap::new();
	let mut met_rec : meteo::Record = HashMap::new();
    
    loop {
        let is_new_block = is_new_epoch(&line, &header);
        if is_new_block && !first {
            match &header.rinex_type {
                Type::NavigationMessage => {
                    if let Ok((e, sv, map)) = navigation::build_record_entry(&header, &block) {
                        let mut smap : HashMap<Sv, HashMap<String, navigation::ComplexEnum>> = HashMap::with_capacity(1);
                        smap.insert(sv, map);
                        nav_rec.insert(e, smap);
                    }
                },
                Type::ObservationData => {
                    if let Ok((e, offset, map)) = observation::build_record_entry(&header, &block) {
                        obs_rec.insert(e, (offset, map));
                    }
                },
				Type::MeteorologicalData => {
					if let Ok((e, map)) = meteo::build_record_entry(&header, &block) {
						met_rec.insert(e, map);
					}
				},
            }
        }

        if is_new_block {
            if first {
                first = false
            }
            block.clear()
        }

        block.push_str(&line);
        block.push_str("\n");

        if eof {
            break
        }

        if let Some(l) = body.next() {
            line = l
        } else {
            eof = true;
        }

        while is_comment!(line) {
            if let Some(l) = body.next() {
                line = l
            } else {
                eof = true; 
            }
        }
    }
    match &header.rinex_type {
        Type::NavigationMessage => Ok(Record::NavRecord(nav_rec)),
        Type::ObservationData => Ok(Record::ObsRecord(obs_rec)), 
		Type::MeteorologicalData => Ok(Record::MeteoRecord(met_rec)),
        //_ => Err(TypeError::UnknownType(header.rinex_type.to_string())),
    }
}
