//! record.rs describes `RINEX` file content
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

use crate::epoch;
use crate::header;
use crate::navigation;
use crate::observation;
use crate::epoch::Epoch;
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
    NavRecord(navigation::Record),
    ObsRecord(observation::Record),
    MeteoRecord(HashMap<Epoch, HashMap<String, f32>>),
}

impl Record {
    /// Returns navigation record
    pub fn as_nav (&self) -> Option<&navigation::Record> {
        match self {
            Record::NavRecord(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_obs (&self) -> Option<&observation::Record> {
        match self {
            Record::ObsRecord(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_meteo (&self) -> Option<&HashMap<Epoch, HashMap<String, f32>>> {
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

/// Returns true if a new block record section
/// is encountered at given line.
/// Block record section represents an epoch for most RINEX files
fn block_record_start (line: &str, header: &header::RinexHeader) -> bool {
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
	match header.version.major {
		1|2|3 => {
			// old RINEX
			// epoch block is type dependent
			match &header.rinex_type {
				Type::NavigationMessage => {
					// old NAV: epoch block
					//  is constellation dependent
					match &header.constellation {
						Constellation::Glonass => {
							// GLONASS NAV special case
							//  constellation ID is implied
							parsed.len() > 4
						},
						_ => {
							// other constellations
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
					}
				},
				Type::ObservationData => {
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
				_ => false, // non supported, 
					// unknown RINEX types
			}
		},
		_ => {
			// modern V > 3 RINEX
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

    let mut nav_rec : HashMap<Epoch, HashMap<Sv, HashMap<String, navigation::ComplexEnum>>> = HashMap::new();
    let mut obs_rec : HashMap<Epoch, HashMap<Sv, HashMap<String, f32>>> = HashMap::new();
    
    loop {
        let is_new_block = block_record_start(&line, &header);
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
                    if let Ok((e, map)) = observation::build_record_entry(&header, &block) {
                        obs_rec.insert(e, map);
                    }
                },
                _ => {},
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

        if let Some(l) = body.next() {
            line = l
        } else {
            break
        }

        while is_comment!(line) {
            if let Some(l) = body.next() {
                line = l
            } else {
                eof = true; 
                break 
            }
        }

        if eof {
            break
        }
    }
    match &header.rinex_type {
        Type::NavigationMessage => Ok(Record::NavRecord(nav_rec)),
        Type::ObservationData => Ok(Record::ObsRecord(obs_rec)), 
        _ => Err(TypeError::UnknownType(header.rinex_type.to_string())),
    }
}
