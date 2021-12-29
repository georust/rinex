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
}

impl Default for NavigationMsgType {
    /// Builds a default `NavigationMsgType`
    fn default() -> NavigationMsgType {
        NavigationMsgType::Cnav
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
/// constellation: GNSS for this particular frame,
///       identical accross entire file for unique RINEX NAV files.   
/// sv_id: Sat. Vehicule ID#   
/// epoch: epoch time stamp    
/// sv_clock_bias: (s)   
/// sv_clock_drift: (s.s⁻¹)   
/// sv_clock_drift_rate: (s.s⁻²)
#[derive(Debug)]
pub struct NavigationRecord {
    record_type: NavigationRecordType,
    msg_type: NavigationMsgType,
    constellation: constellation::Constellation,
    sv_id: u8, // Vehicule #ID 
    epoch: chrono::NaiveDateTime, // timestamp
    // orbit items (constellation + vers. dependent)
    // SV#ID , epoch, SVClock (bias (s), drift (s.s⁻¹) rate (s.s⁻²)
    items: std::collections::HashMap<String, RecordItem>,
}

impl Default for NavigationRecord {
    fn default() -> NavigationRecord {
        NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            constellation: constellation::Constellation::default(),
            sv_id: 0,
            epoch: chrono::NaiveDate::from_ymd(2000,01,01)
                .and_hms(0,0,0),
            items: std::collections::HashMap::with_capacity(MaxMapSize),
        }
    }
}

impl NavigationRecord {
    /// Builds date (timestamp) from raw str items
    fn parse_date (items: &[&str]) -> Result<chrono::NaiveDateTime, Error> {
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
        Ok(chrono::NaiveDate::from_ymd(y,mon,day)
            .and_hms(h,min,s as u32))
    }
    
    /// Builds `NavigationRecord` from raw record content
    pub fn from_string (version: &version::Version, constellation: &constellation::Constellation, s: &str) -> Result<NavigationRecord, Error> {
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
        
        // line 1 always contains 
        // [+] SV#ID
        // [+] time stamp
        // [+] sv clock bias
        // [+] sv clock drift
        // [+] sv clock drift rate
        let (sat_id_and_date, rem) = line.split_at(22);
        let (bias, rem) = rem.split_at(19);
        let (drift, rem) = rem.split_at(19);
        let (drift_rate, rem) = rem.split_at(19);

        let items: Vec<&str> = sat_id_and_date.split_ascii_whitespace()
            .collect();

        let nav_message_known_sv_identifiers: &'static [char] =
            &['R','G','E','B','J','C','S']; 

        let (constel, sv_id): (constellation::Constellation, u8) = match nav_message_known_sv_identifiers
                .contains(&items[0].chars().nth(0).unwrap()) 
        {
            true => {
                // V > 3 contains satid#PRN, and not PRN only
                (constellation::Constellation::from_str(items[0])?,
                u8::from_str_radix(&items[0][1..], 10)?)
            },
            false => {
                match constellation {
                    constellation::Constellation::Glonass => {
                        // Glonass dedicated NAV + old fashion: no 'G'
                        (constellation::Constellation::from_str(items[0])?,
                        u8::from_str_radix(&items[0], 10)?)
                    },
                    constellation::Constellation::Mixed => {
                        // Mixed requires 'ID' + PRN#
                        (constellation::Constellation::from_str(items[0])?,
                        u8::from_str_radix(&items[0][1..], 10)?)
                    },
                    c => (*c, u8::from_str_radix(&items[0], 10)?)
                }
            }
        };

        let epoch = NavigationRecord::parse_date(&items[1..7])?;
        //println!("BIAS \"{}\" DRIFT \"{}\" RATE \"{}\"",&bias.replace("D","e"),drift,drift_rate);
        //let sv_clock_bias = 0.0_f64; //f64::from_str(&bias.replace("D","e"))?;
        //let sv_clock_drift = 0.0_f64; //f64::from_str(&drift.replace("D","e"))?;
        //let sv_clock_drift_rate = 0.0_f64; //f64::from_str(&drift_rate.replace("D","e"))?;

        let mut map = std::collections::HashMap::with_capacity(MaxMapSize);
        let item = RecordItem::from_string(RecordItemType::Float64, bias)?; 
        map.insert(String::from("SvClockBias"), item);
        //map.insert(String::from("SvClockDrift"), sv_clock_drift);
        //map.insert(String::from("SvClockDriftRate"), sv_clock_drift_rate);
        //map.insert(String::from("Epoch"), epoch);

        // orbits parsing
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
            epoch,
            items: map, 
        })
    }
}
