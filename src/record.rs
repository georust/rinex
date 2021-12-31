//! record.rs describes RINEX file content

use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

use crate::keys;
use crate::RinexType;
use crate::version::RinexVersion;
use crate::constellation::Constellation;
use crate::navigation::*; 

pub use crate::navigation::{NavigationRecordType, NavigationMsgType};

/// Record describes Rinex File content.    
/// A record entry is a hashmap.
pub type RinexRecord = Vec<HashMap<String, RecordItem>>;

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("rinex file \"{0}\" is not supported")]
    NonSupportedRinexType(String),
    #[error("failed to build record item")]
    RecordItemError(#[from] RecordItemError),
}

/// `Epoch` is the timestamp of an observation
type Epoch = chrono::NaiveDateTime;

/// ̀`Sv` describes a Satellite Vehicule
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Sv {
    constellation: Constellation,
    prn: u8,
}

/// ̀ Sv` parsing / identification related errors
#[derive(Error, Debug)]
pub enum SvError {
    #[error("unknown constellation marker \"{0}\"")]
    UnidentifiedConstellation(char),
    #[error("failed to parse Sv #PRN")]
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
    /// Creates a new `Sv` Satellite vehicule descriptor
    pub fn new (constellation: Constellation, prn: u8) -> Sv { Sv {constellation, prn }}

    /// Returns `GNSS` constellation from which this
    /// `Sv` is part of
    pub fn get_constellation (&self) -> Constellation { self.constellation }

    /// Returns `PRN#ID` of this particular `Sv`
    pub fn get_prn (&self) -> u8 { self.prn }
}

impl std::str::FromStr for Sv {
    type Err = SvError;
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

#[derive(Copy, Clone, PartialEq, Debug)]
/// `RecordItem` describes all known Rinex Records items
pub enum RecordItem {
    Sv(Sv),
    Float64(f64),
    Epoch(Epoch),
    Unsigned(u32),
    Flag(u8),
    // (NAV)
    NavRecType(NavigationRecordType),
    NavMsgType(NavigationMsgType),
}

/*
impl PartialEq for RecordItem {
    fn eq (&self, other: &Self) -> bool {
        if let RecordItem::Sv(s) = self {
            if let RecordItem::Sv(o) = other {
                return s.get_constellation() == o.get_constellation()
            }
        }
        false
    }
}

impl Eq for RecordItem {} */

#[derive(Error, Debug)]
/// `RecordItem` related errors
pub enum RecordItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
    #[error("failed to parse sv")]
    SvParsingError(#[from] SvError), 
}

impl RecordItem {
    /// Builds a `RecordItem` from type descriptor and string content
    pub fn from_string (type_descriptor: &str, content: &str) 
            -> Result<RecordItem, RecordItemError> 
    {
        //println!("Building \'{}\' from \"{}\"", type_descriptor, content);
        match type_descriptor {
            //TODO
            // normalement pas besoin du replace D->E pour f64
            // introduire un type fixed point (scaled integer)
            //TODO
            // un type binary peut aider pour les mask..
            // u32 doit suffir
            "sv" => Ok(RecordItem::Sv(Sv::from_str(&content)?)),
            "d19.12" => Ok(RecordItem::Float64(f64::from_str(&content.replace("D","e"))?)),
             "epoch" => {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                let (mut y,mon,day,h,min,s): (i32,u32,u32,u32,u32,f64) =
                    (i32::from_str_radix(items[0], 10)?,
                    u32::from_str_radix(items[1], 10)?,
                    u32::from_str_radix(items[2], 10)?,
                    u32::from_str_radix(items[3], 10)?,
                    u32::from_str_radix(items[4], 10)?,
                    f64::from_str(items[5])?);
                if y < 100 {
                    y += 2000 // 2 digit nb case
                }
                Ok(RecordItem::Epoch(
                    chrono::NaiveDate::from_ymd(y,mon,day)
                        .and_hms(h,min,s as u32)))
            },
            "i1" => Ok(RecordItem::Flag(u8::from_str_radix(&content, 10)?)),
            "navRecType" => Ok(RecordItem::NavRecType(NavigationRecordType::from_str(&content)?)),
            "navMsgType" => Ok(RecordItem::NavMsgType(NavigationMsgType::from_str(&content)?)),
            _ => Err(RecordItemError::UnknownTypeDescriptor(type_descriptor.to_string())),
        }
    }
}

/// Identifies starting point of a new block record content,
/// from which we will build a RinexRecord entry afterwards 
pub fn block_record_start (line: &str,
    rinex_type: &RinexType,
        constellation: &Constellation, 
            version: &RinexVersion) -> bool
{
    let major = version.get_major();
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
    
    match major < 4 {
        true => {
            // RinexType:: dependent
            match rinex_type {
                RinexType::NavigationMessage => {
                    let known_sv_identifiers: &'static [char] = 
                        &['R','G','E','B','J','C','S']; 
                    match constellation {
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

/// Builds a record entry from block record content.   
/// Entry of a record is a hashmap
pub fn build_record_entry (version: RinexVersion, 
        rtype: RinexType, 
            constellation: Constellation, block: &str) 
                -> Result<HashMap<String,RecordItem>, RecordError> 
{
    match rtype {
        RinexType::NavigationMessage => {
            let entry = build_nav_entry(version, constellation, block)?;
            Ok(entry)
        },
        _ => Err(RecordError::NonSupportedRinexType(rtype.to_string())), 
    }
}
