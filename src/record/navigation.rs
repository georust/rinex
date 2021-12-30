//! Navigation.rs
//! to describe `Rinex` file body content
//! for NavigationMessage files
use thiserror::Error;
use chrono::Timelike;
use std::str::FromStr;

use crate::record::*;
use crate::record::keys::*;
use crate::header::RinexType;
use crate::version::RinexVersion;
use crate::constellation::{Constellation, ConstellationError};

/// Describes NAV records specific errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify constellation")]
    ParseConstellationError(#[from] ConstellationError),
    #[error("failed to build record item")]
    RecordItemError(#[from] RecordItemError),
    #[error("satellite vehicule parsing error")]
    SvError(#[from] SvError),
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
///  Lnav: Legacy NAV Messsage   
///  Cnav: Civilian NAV Messsage   
///  Cnav2: Civilian NAV Messsage   
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
        NavigationMsgType::Lnav
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
    /// Builds a default `NavigationRecord`
    fn default() -> NavigationRecord {
        NavigationRecord {
            record_type: NavigationRecordType::default(),
            msg_type: NavigationMsgType::default(),
            items: std::collections::HashMap::with_capacity(RECORD_MAX_SIZE),
        }
    }
}

impl NavigationRecord {
    /// Builds `NavigationRecord` from raw record content
    pub fn from_string (version: RinexVersion, 
            constellation: Constellation, s: &str) 
                -> Result<NavigationRecord, Error> 
    {
        // NAV data is partially flexible
        //  <o 
        //     SV + Epoch + SvClock infos + RecType + MsgType are always there
        //     Other items are constellation dependent => key map
        //     easier to deal with than OBS: 
        //           (*) listing is fixed
        //           (*) nb of items fixed
        let mut lines = s.lines();

        let mut msg_type = NavigationMsgType::default();
        let mut record_type = NavigationRecordType::default();
        let mut map: std::collections::HashMap<String, RecordItem> 
            = std::collections::HashMap::with_capacity(RECORD_MAX_SIZE);

        let version_major = version.get_major();

        let mut line = lines.next()
            .unwrap();
        // we might parse a 1st line
        // V >= 4
        //    [+] RecType + SV + MsgType 
        //                       ^---> newly introduced
        //                   ^-------> deduce constellation identification keys
        //                             as we didn't get this information from file header
        //         ^-----------------> newly introduced
        let (rec_type, sv, msg_type): (NavigationRecordType, Option<Sv>, NavigationMsgType)
                = match version_major >= 4 
        {
            true => {
                let items: Vec<&str> = line.split_ascii_whitespace()
                    .collect();
                let rec_type = NavigationRecordType::from_str(&items[0])?;
                let sv = Some(Sv::from_str(items[1].trim())?);
                let msg_type = NavigationMsgType::from_str(&items[2])?; 
                line = lines.next()
                    .unwrap();
                (rec_type,sv,msg_type)
            },
            false => (NavigationRecordType::default(), None, NavigationMsgType::default()),
        };

        // we might parse a 2nd line
        // V < 4
        //    [+] SV + Epoch + SvClock infos
        //         ^-> deduce constellation identification keys
        //             as we didn't get this information from file header
        // V >= 4
        //    [+]  Epoch ; SvClock infos
        if sv.is_none() {
            let (sv, rem) = line.split_at(3);
            let (epoch, rem) = rem.split_at(20);
            let (svbias, rem) = rem.split_at(19);
            let (svdrift, svdriftr) = rem.split_at(19);

            let sv: Sv = match constellation {
                // SV problem
                //  (+) GLONASS NAV special case
                //      SV'X' is not implied
                //  (+) faulty RINEX producer with unique constellation
                //      SV'X' is dropped => deal with that
                Constellation::Mixed => Sv::from_str(sv.trim())?,
                _ => {
                    let prn = u8::from_str_radix(sv.trim(), 10)?;  
                    Sv::new(constellation, prn)
                }
            };
            let sv = RecordItem::Sv(sv);
            let epoch = RecordItem::from_string("epoch", epoch.trim())?;
            let clk_bias = RecordItem::from_string("d19.12", svbias.trim())?;
            let clk_drift = RecordItem::from_string("d19.12", svdrift.trim())?;
            let clk_drift_r = RecordItem::from_string("d19.12", svdriftr.trim())?;
            map.insert(String::from("sv"), sv); 
            map.insert(String::from("epoch"), epoch); 
            map.insert(String::from("svClockBias"), clk_bias); 
            map.insert(String::from("svClockDrift"), clk_drift); 
            map.insert(String::from("svClockDriftRate"), clk_drift_r); 
        }

        // from now one, everything is mapped
        // as it is fixed and constant
        // but it depends on the rinex context (release, constellation)
        let kbank = KeyBank::new(&version, &RinexType::NavigationMessage, &constellation)
            .unwrap();

        let mut total: usize = 0; 
        let mut new_line = true;

        line = lines.next()
            .unwrap();

        for key in &kbank.keys { 
            let (k_name, k_type) = key; 
            let offset: usize = match new_line {
                false => 19,
                true => {
                    new_line = false;
                    22
                }
            };
            total += offset;
            let (content, rem) = line.split_at(offset); 
            line = rem;

            // build item 
            let item = RecordItem::from_string(k_type, content.trim())?;
            map.insert(String::from(k_name), item); 

            if total >= 76 { 
                new_line = true;
                total = 0;
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
