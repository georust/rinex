//! `RinexType::NavigationMessage` specific module
use chrono::Timelike;
use std::str::FromStr;
use std::collections::HashMap;

use crate::RinexType;
use crate::keys::*;
use crate::version::RinexVersion;
use crate::record::{RecordItem, Sv, RecordItemError};
use crate::constellation::{Constellation, ConstellationError};

#[derive(Copy, Clone, PartialEq, Debug)]
/// `NavigationRecordType` describes type of record
/// for NAV files
pub enum NavigationRecordType {
    Ephemeride,
    Ionospheric,
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
        if s.contains("EPH") {
            Ok(NavigationRecordType::Ephemeride)
        } else if s.contains("ION") {
            Ok(NavigationRecordType::Ionospheric)
        } else {
            Ok(NavigationRecordType::Ephemeride)
        }
    }
}

impl NavigationRecordType {
    /// Converts Self to &str
    fn to_string (&self) -> &str {
        match self {
            NavigationRecordType::Ephemeride  => "EPH",
            NavigationRecordType::Ionospheric => "ION",
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
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

impl NavigationMsgType {
    /// Converts Self to &str
    fn to_string (&self) -> &str {
        match self {
            NavigationMsgType::Lnav =>  "LNAV",
            NavigationMsgType::Cnav =>  "CNAV",
            NavigationMsgType::Cnav2 => "CNAV2",
            NavigationMsgType::Inav =>  "INAV",
            NavigationMsgType::Fnav =>  "FNAV",
            NavigationMsgType::Fdma =>  "FDMA",
        }
    }
}

/// Builds a RinexType::NavigationMessage specific record entry, 
/// from given string content
pub fn build_nav_entry (version: RinexVersion, 
    constellation: Constellation, content: &str) 
        -> Result<HashMap<String, RecordItem>, RecordItemError> 
{
    // NAV 
    //  <o 
    //     SV + Epoch + SvClock infos + RecType + MsgType are always there
    //     Other items are constellation dependent => key map
    //     easier to deal with than OBS: 
    //           (*) listing is fixed
    //           (*) nb of items fixed
    let mut lines = content.lines();

    let mut msg_type = NavigationMsgType::default();
    let mut record_type = NavigationRecordType::default();
    let mut map: HashMap<String, RecordItem> 
        = std::collections::HashMap::with_capacity(KEY_BANK_MAX_SIZE);

    let version_major = version.get_major();

    let mut line = lines.next()
        .unwrap();
    
    // might parse a 1st line (V ≥ 4)
    //    [+] RecType + SV + MsgType 
    //                       ^---> newly introduced
    //                   ^-------> deduce constellation identification keys
    //                             as we didn't get this information from file header
    //         ^-----------------> newly introduced
    let (rectype, sv_str, msgtype): (RecordItem, Option<&str>, RecordItem)
            = match version_major >= 4 
    {
        true => {
            let items: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            line = lines.next()
                .unwrap();
            (RecordItem::from_string("navRecType", &items[0])?,
            Some(items[1].trim()),
            RecordItem::from_string("navMsgType", &items[2])?)
        },
        false => (RecordItem::from_string("navRecType", NavigationRecordType::default().to_string())?, 
            None, 
            RecordItem::from_string("navMsgType", NavigationMsgType::default().to_string())?),
    };

    map.insert(String::from("navRecType"), rectype);
    map.insert(String::from("navMsgType"), msgtype);

    // parse a 2nd line
    // V < 4
    //    [+] SV + Epoch + SvClock infos
    //         ^-> deduce constellation identification keys
    //             as we didn't get this information from file header
    // V ≥ 4
    //    [+]  Epoch ; SvClock infos
    let (sv_str, rem) : (&str, &str) = match sv_str.is_none() {
        true => line.split_at(3), // V < 4
        false => (sv_str.unwrap(), line), // V ≥ 4
    };

    let (epoch, rem) = rem.split_at(20);
    let (svbias, rem) = rem.split_at(19);
    let (svdrift, svdriftr) = rem.split_at(19);

    let sv: RecordItem = match constellation {
        // SV identification problem
        //  (+) GLONASS NAV special case
        //      SV'X' is omitted 
        //  (+) faulty RINEX producer with unique constellation
        //      SV'X' is omitted
        Constellation::Mixed => RecordItem::from_string("sv", sv_str.trim())?,
        _ => {
            let prn = u8::from_str_radix(sv_str.trim(), 10)?;  
            RecordItem::Sv(Sv::new(constellation, prn))
        },
    };
    map.insert(String::from("sv"), sv); 

    let epoch = RecordItem::from_string("epoch", epoch.trim())?;
    let clk_bias = RecordItem::from_string("d19.12", svbias.trim())?;
    let clk_drift = RecordItem::from_string("d19.12", svdrift.trim())?;
    let clk_drift_r = RecordItem::from_string("d19.12", svdriftr.trim())?;
    map.insert(String::from("epoch"), epoch); 
    map.insert(String::from("svClockBias"), clk_bias); 
    map.insert(String::from("svClockDrift"), clk_drift); 
    map.insert(String::from("svClockDriftRate"), clk_drift_r); 
    
    // from now one, everything is described in key mapping
    //   ---> refer to Sv identified constell,
    //        because we simply cannot search for "Mixed"
    let kbank = KeyBank::new(&version, &RinexType::NavigationMessage, &sv.Sv().unwrap().get_constellation())
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
                if version_major >= 3 {
                    22 + 1
                } else {
                    22
                }
            }
        };
        total += offset;
        let (content, rem) = line.split_at(offset); 
        line = rem;

        // build item 
        if !k_type.eq("spare") {
            let item = RecordItem::from_string(k_type, content.trim())?;
            map.insert(String::from(k_name), item); 
        }

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
    Ok(map)
}
