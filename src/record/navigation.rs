//! Navigation.rs
//! to describe `Rinex` file body content
//! for NavigationMessage files
use thiserror::Error;
use chrono::Timelike;
use std::str::FromStr;

use crate::keys;
use crate::record;
use crate::version;
use crate::constellation;

use record::*;

/// Describes NAV records specific errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify constellation")]
    ParseConstellationError(#[from] constellation::ConstellationError),
    #[error("failed to build record item")]
    RecordItemError(#[from] RecordItemError),
}

#[derive(Debug)]
/// `NavigationRecordType` describes type of record
/// for NAV files
enum NavigationRecordType {
    Ephemeride,
}

impl Default for NavigationRecordType {
    /// Builds a default `NavigationRecordType`
    fn default() -> NavigationRecordType {
        NavigationRecordType::Ephemeride
    }
}

impl std::str::FromStr for NavigationRecordType {
    type Err = std::num::ParseIntError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("LNAV") {
            Ok(NavigationRecordType::Ephemeride)
        } else {
            Ok(NavigationRecordType::Ephemeride)
        }
    }
}

#[derive(Debug)]
/// `NavigationMsgType`
/// describes messages type for NAV files
pub enum NavigationMsgType {
    Lnav,
    Cnav,
    Cnav2,
    Fdma,
    Inav,
    Fnav,
    Unknown,
}

impl Default for NavigationMsgType {
    /// Builds a default `NavigationMsgType`
    fn default() -> NavigationMsgType {
        NavigationMsgType::Unknown
    }
}

impl std::str::FromStr for NavigationMsgType {
    type Err = std::num::ParseIntError;
    /// Builds a `NavigationMsgType` from a string
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains("LNAV") {
            Ok(NavigationMsgType::Lnav)
        } else if s.contains("CNAV") {
            Ok(NavigationMsgType::Cnav) 
        } else if s.contains("CNAV2") {
            Ok(NavigationMsgType::Cnav2)
        } else if s.contains("INAV") {
            Ok(NavigationMsgType::Inav)
        } else if s.contains("FNAV") {
            Ok(NavigationMsgType::Fnav)
        } else if s.contains("FDMA") {
            Ok(NavigationMsgType::Fdma)
        } else {
            Ok(NavigationMsgType::Cnav)
        }
    }
}

/// `NavigationRecord` describes a NAV message frame.   
/// constellation: GNSS for this particular record,
///       identical accross entire file for unique RINEX NAV files.   
/// items: collection of record items   
///    SV#ID, epoch, SvClockBias (s), SvClockDrift (s.s⁻¹) SvClockDriftRate (s.s⁻²), ...
#[derive(Debug)]
pub struct NavigationRecord {
    record_type: NavigationRecordType,
    msg_type: NavigationMsgType,
    items: std::collections::HashMap<String, RecordItem>,
}

impl Default for NavigationRecord {
    fn default() -> NavigationRecord {
        NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            items: std::collections::HashMap::with_capacity(keys::KEY_BANK_MAX_SIZE),
        }
    }
}

impl NavigationRecord {
    /// Builds `NavigationRecord` from raw record content
    pub fn from_string (version: version::Version, 
            constellation: constellation::Constellation, 
                key_listing: &keys::KeyBank, s: &str) 
                -> Result<NavigationRecord, Error> 
    {
        let mut lines = s.lines();
        let mut line = lines.next()
            .unwrap();

        let mut msg_type = NavigationMsgType::default();
        let mut record_type = NavigationRecordType::default();
        if version.get_major() > 3 {
            let items: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            record_type = NavigationRecordType::from_str(&items[3])?; 
            msg_type = NavigationMsgType::from_str(&items[1])?;
            line = lines.next()
                .unwrap()
        }

        let mut item_count: u8 = 0;
        let mut map = std::collections::HashMap::with_capacity(keys::KEY_BANK_MAX_SIZE);

        for key in &key_listing.keys { 
            let (key, type_descriptor) = key; 

            let offset: usize = match type_descriptor.as_str() {
                "sv" => 3,
                _ => 19,
            };

            let (content, rem) = line.split_at(offset); 
            line = rem.trim();
            let content = content.trim(); 
            let mut item = RecordItem::from_string(type_descriptor, content)?;

            // special case: GLONASS NAV
            // special case: faulty Rinex producer
            //   GLONASS NAV (ok) and faulty producers
            //   do not prepend 'Sv' identifier with constellation
            //   identifier
            //   -> manual assignment in this case
            if type_descriptor.eq("sv") && constellation != constellation::Constellation::Mixed {
                let prn = u8::from_str_radix(&content, 10)?;  
                item = RecordItem::Sv(Sv::new(constellation, prn))
            }

            map.insert(String::from(key), item); 

            if map.len() % 4 == 0 {
                // time to grab a new line
                println!("new line");
                if let Some(l) = lines.next() {
                    line = l;   
                } else {
                    break
                }
            }
        }

        Ok(NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            items: map, 
        })
    }
}
