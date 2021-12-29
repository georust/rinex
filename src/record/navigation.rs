//! Navigation.rs
//! to describe `Rinex` file body content
//! for NavigationMessage files
use thiserror::Error;
use chrono::Timelike;
use std::str::FromStr;

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

/// Maximal nb of items to be stored 
pub const MaxMapSize: usize = 16;

/// `NavigationRecord` describes a NAV message frame.   
/// constellation: GNSS for this particular record,
///       identical accross entire file for unique RINEX NAV files.   
/// items: collection of record items   
///    SV#ID, epoch, SvClockBias (s), SvClockDrift (s.s⁻¹) SvClockDriftRate (s.s⁻²), ...
#[derive(Debug)]
pub struct NavigationRecord {
    record_type: NavigationRecordType,
    msg_type: NavigationMsgType,
    constellation: constellation::Constellation,
    sv_id: u8, // Vehicule #ID 
    items: std::collections::HashMap<String, RecordItem>,
}

impl Default for NavigationRecord {
    fn default() -> NavigationRecord {
        NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            constellation: constellation::Constellation::default(),
            sv_id: 0,
            items: std::collections::HashMap::with_capacity(MaxMapSize),
        }
    }
}

impl NavigationRecord {
    /// Builds `NavigationRecord` from raw record content
    pub fn from_string (version: &version::Version, 
            constellation: &constellation::Constellation, s: &str) 
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

        //let keys: Vec<String>  match version.get_major() {
            //match constellation {
            //}
        //};
        
        // line 1 always contains 
        // [+] SV#ID
        // [+] epoch 
        // [+] sv clock bias
        // [+] sv clock drift
        // [+] sv clock drift rate
        let (sat_id, rem) = line.split_at(3);
        let (date, rem) = rem.split_at(19);
        let (bias, rem) = rem.split_at(19);
        let (drift, rem) = rem.split_at(19);
        let (drift_rate, rem) = rem.split_at(19);

        let nav_message_known_sv_identifiers: &'static [char] =
            &['R','G','E','B','J','C','S']; 

        let mut map = std::collections::HashMap::with_capacity(MaxMapSize);
        
        let item = RecordItem::from_string("epoch", date)?; 
        map.insert(String::from("Epoch"), item);
        
        let item = RecordItem::from_string("f64", bias.trim())?; 
        map.insert(String::from("SvClockBias"), item);
        
        let item = RecordItem::from_string("f64", drift.trim())?; 
        map.insert(String::from("SvClockDrift"), item);
        
        let item = RecordItem::from_string("f64", drift_rate.trim())?; 
        map.insert(String::from("SvClockDriftRate"), item);

        let (constel, sv_id): (constellation::Constellation, u8) = match nav_message_known_sv_identifiers
                .contains(&sat_id.trim().chars().nth(0).unwrap()) 
        {
            true => {
                // V ≥ 4 contains #Sat#PRN, not PRN only
                (constellation::Constellation::from_char(sat_id.chars().nth(0).unwrap())?,
                u8::from_str_radix(&sat_id.trim()[1..], 10)?)
            },
            false => {
                // V < 4 contains #Sat#PRN, or ' '#PRN
                match constellation {
                    constellation::Constellation::Glonass => {
                        // Glonass dedicated NAV + old fashion: no 'G'
                        (constellation::Constellation::Glonass,
                        u8::from_str_radix(&sat_id.trim(), 10)?)
                    },
                    constellation::Constellation::Mixed => {
                        // Mixed requires 'ID' + PRN#
                        (constellation::Constellation::from_char(sat_id.chars().nth(0).unwrap())?,
                        u8::from_str_radix(&sat_id.trim()[1..], 10)?)
                    },
                    c => (*c, u8::from_str_radix(&sat_id.trim(), 10)?)
                }
            }
        };

        loop {
            if let Some(l) = lines.next() {
                line = l;   
            } else {
                break
            }
            let (data1, rem) = line.split_at(23);
            let (data2, rem) = rem.split_at(19);
            let (data3, rem) = rem.split_at(19);
            let (data4, rem) = rem.split_at(19);
        }
        
        Ok(NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            constellation: constel,
            sv_id,
            items: map, 
        })
    }
}
