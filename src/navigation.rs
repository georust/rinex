//! `NavigationMessage` parser and related methods
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

use crate::epoch;
use crate::record;
use crate::version;
use crate::constellation;
use crate::header::RinexHeader;
use crate::record::{Sv, ComplexEnum};

include!(concat!(env!("OUT_DIR"),"/nav_data.rs"));

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("failed to parse msg type")]
    ParseSvError(#[from] record::ParseSvError),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] record::ComplexEnumError),
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
    let mut map: HashMap<String, ComplexEnum> = HashMap::new(); 
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
        constellation::Constellation::Mixed => Sv::from_str(sv.trim())?,
        _ => {
            let prn = u8::from_str_radix(sv.trim(), 10)?;  
            Sv::new(header.constellation, prn)
        },
    };

    map.insert("ClockBias".into(), ComplexEnum::new("f32", svbias.trim())?); 
    map.insert("ClockDrift".into(), ComplexEnum::new("f32", svdrift.trim())?); 
    map.insert("ClockDriftRate".into(), ComplexEnum::new("f32", svdriftr.trim())?); 

    line = lines.next()
        .unwrap();

    let mut total: usize = 0; 
    let mut new_line = true;
    
    // database
    let db : Vec<_> = NAV_MESSAGES.iter().collect();
    for nav in db {
        let to_match = constellation::Constellation::from_str(nav.constellation)
            .unwrap();
        //   ---> refer to Sv identified constell,
        //        because header.constellation might be `Mixed`
        if sv.constellation == to_match {
            for rev in nav.revisions.iter() {
                let major = u8::from_str_radix(rev.major,10)
                    .unwrap();
                // TODO:
                // improve revision matching
                // minor should be used: use closest revision
                //let minor = u8::from_str_radix(rev.minor,10)
                //    .unwrap();
                if major == header.version.major {
                    for item in rev.items.iter() {
                        let (k, v) = item;
                        let offset: usize = match new_line {
                            false => 19,
                            true => {
                                new_line = false;
                                if header.version.major >= 3 {
                                    22+1
                                } else {
                                    22
                                }
                            }
                        };
                        total += offset;
                        let (content, rem) = line.split_at(offset);
                        line = rem;
                        if !k.eq(&"spare") {
                            let rec_item = ComplexEnum::new(v, content.trim())?;
                            map.insert(String::from(*k), rec_item);
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
                }
            }
        }
    }
    Ok((epoch::from_string(epoch)?, sv, map))
}

mod test {
    use super::*;
    #[test]
    /// Tests static NAV database to be used in dedicated parser
    fn test_nav_database() {
        for n in NAV_MESSAGES.iter() { 
            constellation::Constellation::from_str(
                n.constellation
            ).unwrap();
            for r in n.revisions.iter() {
                u8::from_str_radix(r.major, 10).unwrap();
                u8::from_str_radix(r.major, 10).unwrap();
                for item in r.items.iter() {
                    let (k, v) = item;
                    if !k.eq(&"spare") {
                        let test : String;
                        if v.eq(&"f32") {
                            test = String::from("0.0")
                        } else if v.eq(&"f64") {
                            test = String::from("0.0")
                        } else if v.eq(&"u8") {
                            test = String::from("10")
                        } else {
                            test = String::from("hello")
                        }
                        ComplexEnum::new(v, &test).unwrap();
                    }
                }
            }
        }
    }
}
