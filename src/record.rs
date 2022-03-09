//! record.rs describes `RINEX` file content
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

use crate::header;
use crate::navigation;
use crate::epoch::Epoch;
use crate::is_rinex_comment;
use crate::{RinexType, RinexTypeError};
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
        let mut prn: u8 = 0;
        let mut constellation = Constellation::default();
        if s.starts_with('G') {
            constellation = Constellation::GPS;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('E') {
            constellation = Constellation::Galileo;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('R') {
            constellation = Constellation::Glonass;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('S') {
            constellation = Constellation::Sbas;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('J') {
            constellation = Constellation::QZSS;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('C') {
            constellation = Constellation::Beidou;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else {
            prn = u8::from_str_radix(&s, 10)?;
        }
        Ok(Sv{constellation, prn})
    }
}

/// `Record`
#[derive(Clone, Debug)]
pub enum Record {
    NavRecord(HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>>),
    ObsRecord(HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>>),
    MeteoRecord(HashMap<Epoch, HashMap<String, ComplexEnum>>),
}

impl Record {
    /// Returns navigation record
    pub fn as_nav (&self) -> Option<&HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>>> {
        match self {
            Record::NavRecord(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_obs (&self) -> Option<&HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>>> {
        match self {
            Record::ObsRecord(e) => Some(e),
            _ => None,
        }
    }
    pub fn as_meteo (&self) -> Option<&HashMap<Epoch, HashMap<String, ComplexEnum>>> {
        match self {
            Record::MeteoRecord(e) => Some(e),
            _ => None,
        }
    }
}

impl Default for Record {
    fn default() -> Record {
        let r : HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>> = HashMap::new();
        Record::NavRecord(r)
    }
}

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("record parsing not supported for type \"{0}\"")]
    TypeError(String),
}

/// Splits block record sections 
fn block_record_start (line: &str, header: &header::RinexHeader) -> bool {
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
    match header.version.major < 4 {
        true => {
            match &header.rinex_type {
                RinexType::NavigationMessage => {
                    let known_sv_identifiers: &'static [char] = 
                        &['R','G','E','B','J','C','S']; 
                    match &header.constellation {
                        Constellation::Glonass => parsed.len() > 4,
                        _ => {
                            match line.chars().nth(0) {
                                Some(c) => known_sv_identifiers.contains(&c), 
                                _ => false
                                    //TODO
                                    // <o 
                                    //   for some files we end up with "\n xxxx" as first frame items 
                                    // current code will discard first payload item in such scenario
                                    // => need to cleanup (split(head,body) method)
                            }
                        }
                    }
                },
                RinexType::ObservationData => parsed.len() > 8,
                _ => false, 
            }
        },
        false => {      
            // V4: OBS blocks have a '>' delimiter
            match line.chars().nth(0) {
                Some(c) => c == '>',
                _ => false,
                    //TODO
                    // <o 
                    //   for some files we end up with "\n xxxx" as first frame items 
                    // current code will discard first payload item in such scenario
                    // => need to cleanup (split(head,body) method)
            }
        },
    }
}

pub fn build_record (header: &header::RinexHeader, body: &str) -> Result<Record, RinexTypeError> { 
    let mut body = body.lines();
    let mut line = body.next()
        .unwrap();
    while is_rinex_comment!(line) {
        line = body.next()
            .unwrap()
    }
    let mut eof = false;
    let mut first = true;
    let mut block = String::with_capacity(256*1024); // max. block size

    let mut rec : HashMap<Epoch, HashMap<Sv, HashMap<String, ComplexEnum>>> = HashMap::new();
    
    loop {
        let parsed: Vec<&str> = line.split_ascii_whitespace()
            .collect();
        
        let is_new_block = block_record_start(&line, &header);
        if is_new_block && !first {
            match &header.rinex_type {
                RinexType::NavigationMessage => {
                    if let Ok((e, sv, map)) = navigation::build_record_entry(&header, &block) {
                        let mut smap : HashMap<Sv, HashMap<String, ComplexEnum>> = HashMap::with_capacity(10);
                        smap.insert(sv, map);
                        rec.insert(e, smap);
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

        while is_rinex_comment!(line) {
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
        RinexType::NavigationMessage => Ok(Record::NavRecord(rec)), 
        RinexType::ObservationData => Ok(Record::ObsRecord(rec)), 
        _ => Err(RinexTypeError::UnknownType(header.rinex_type.to_string())),
    }
}

/// `ComplexEnum` is record payload 
#[derive(Clone, Debug)]
pub enum ComplexEnum {
    U8(u8),
    Str(String), 
    F32(f32),
    F64(f64),
}

/// `ComplexEnum` related errors
#[derive(Error, Debug)]
pub enum ComplexEnumError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
}

impl ComplexEnum {
    /// Builds a `ComplexEnum` from type descriptor and string content
    pub fn new (desc: &str, content: &str) -> Result<ComplexEnum, ComplexEnumError> {
        //println!("Building \'{}\' from \"{}\"", desc, content);
        match desc {
            "f32" => {
                Ok(ComplexEnum::F32(f32::from_str(&content.replace("D","e"))?))
            },
            "f64" => {
                Ok(ComplexEnum::F64(f64::from_str(&content.replace("D","e"))?))
            },
            "u8" => {
                Ok(ComplexEnum::U8(u8::from_str_radix(&content, 16)?))
            },
            "str" => {
                Ok(ComplexEnum::Str(String::from(content)))
            },
            _ => Err(ComplexEnumError::UnknownTypeDescriptor(desc.to_string())),
        }
    }
}
