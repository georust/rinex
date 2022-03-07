//! `RinexType::NavigationMessage` parser & related types
use thiserror::Error;
use std::str::FromStr;

use crate::epoch;
use crate::keys::*;
use crate::RinexType;
use crate::header::RinexHeader;
use crate::version::RinexVersion;
use crate::constellation::Constellation;
use crate::record::{Sv, ComplexEnum};

use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("failed to parse msg type")]
    ParseSvError(#[from] crate::record::ParseSvError),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] crate::record::ComplexEnumError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse epoch")]
    ParseEpochError(#[from] epoch::ParseEpochError), 
}

/// Builds `RinexRecord` entry for `NavigationMessage` file
pub fn build_record_entry (header: &RinexHeader, content: &str)
        -> Result<(epoch::Epoch, Sv, HashMap<String, ComplexEnum>), RecordError>
{
    //  <o 
    //     SV + Epoch + SvClock infos + RecType + MsgType are always there
    //     Other items are constellation dependent => key map
    //     easier to deal with than OBS: 
    //           (*) listing is fixed
    //           (*) nb of items fixed
    let mut lines = content.lines();

    let mut map: HashMap<String, ComplexEnum> 
        = std::collections::HashMap::with_capacity(KEY_BANK_MAX_SIZE);

    let version_major = header.version.major;

    let mut line = lines.next()
        .unwrap();
    
    // might parse a 1st line (V ≥ 4)
    //    [+] RecType + SV + MsgType 
    //                       ^---> newly introduced
    //                   ^-------> deduce constellation identification keys
    //                             as we didn't get this information from file header
    //         ^-----------------> newly introduced
    let (rectype, sv, msgtype): (ComplexEnum, Option<&str>, ComplexEnum)
            = match version_major >= 4 
    {
        true => {
            let items: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            line = lines.next()
                .unwrap();
            (ComplexEnum::new("str", &items[0].trim())?,
            Some(items[1].trim()),
            ComplexEnum::new("str", &items[2].trim())?)
        },
        false => {
            (ComplexEnum::new("str", "EPH")?, // ephemeride, default
            None,
            ComplexEnum::new("str", "LNAV")?) // legacy, default
        },
    };

    map.insert(String::from("msg"), msgtype);
    map.insert(String::from("type"), rectype);

    // parse a 2nd line
    // V < 4
    //    [+] SV + Epoch + SvClock infos
    //         ^-> deduce constellation identification keys
    //             as we didn't get this information from file header
    // V ≥ 4
    //    [+]  Epoch ; SvClock infos
    let (sv, rem) : (&str, &str) = match sv.is_none() {
        true => line.split_at(3), // V < 4
        false => (sv.unwrap(), line), // V ≥ 4
    };

    let (epoch, rem) = rem.split_at(20);
    let (svbias, rem) = rem.split_at(19);
    let (svdrift, svdriftr) = rem.split_at(19);

    let sv: Sv = match header.constellation {
        // SV identification problem
        //  (+) GLONASS NAV special case
        //      SV'X' is omitted 
        //  (+) faulty RINEX producer with unique constellation
        //      SV'X' is omitted
        Constellation::Mixed => Sv::from_str(sv.trim())?,
        _ => {
            let prn = u8::from_str_radix(sv.trim(), 10)?;  
            Sv::new(header.constellation, prn)
        },
    };

    map.insert("ClockBias".into(), ComplexEnum::new("f32", svbias.trim())?); 
    map.insert("ClockDrift".into(), ComplexEnum::new("f32", svdrift.trim())?); 
    map.insert("ClockDriftRate".into(), ComplexEnum::new("f32", svdriftr.trim())?); 
    
    // from now one, everything is described in key mapping
    //   ---> refer to Sv identified constell,
    //        because we simply cannot search for "Mixed"
    /*let kbank = KeyBank::new(&header.version, 
        &sv.as_sv().unwrap().get_constellation())
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
        if !k_name.eq("spare") {
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
    }*/
    //println!("sv: {:#?}, epoch: \"{}\"", sv, epoch);
    Ok((epoch::from_string(epoch)?, sv, map))
}
