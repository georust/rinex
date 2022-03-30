//! `NavigationMessage` parser and related methods
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

use crate::epoch;
use crate::sv;
use crate::record;
use crate::constellation;
use crate::header::Header;

include!(concat!(env!("OUT_DIR"),"/nav_data.rs"));

/// Record definition for NAV files
pub type Record = HashMap<epoch::Epoch, HashMap<sv::Sv, HashMap<String, ComplexEnum>>>;

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
    pub fn as_f32 (&self) -> Option<f32> {
        match self {
            ComplexEnum::F32(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_f64 (&self) -> Option<f64> {
        match self {
            ComplexEnum::F64(f) => Some(f.clone()),
            _ => None,
        }
    }
    pub fn as_str (&self) -> Option<String> {
        match self {
            ComplexEnum::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
    pub fn as_u8 (&self) -> Option<u8> {
        match self {
            ComplexEnum::U8(u) => Some(u.clone()),
            _ => None,
        }
    }
}

#[derive(Error, Debug)]
pub enum RecordError {
    #[error("failed to parse msg type")]
    SvError(#[from] sv::Error),
    #[error("failed to parse cplx data")]
    ParseComplexError(#[from] ComplexEnumError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError), 
}

/// Builds `RinexRecord` entry for `NavigationMessage` file.    
/// `header` : previously parsed Header    
/// `content`: should comprise entire epoch content
pub fn build_record_entry (header: &Header, content: &str)
        -> Result<(epoch::Epoch, sv::Sv, HashMap<String, ComplexEnum>), RecordError>
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

    let (date, rem) = rem.split_at(20);
    let (svbias, rem) = rem.split_at(19);
    let (svdrift, svdriftr) = rem.split_at(19);

    let sv: sv::Sv = match header.constellation {
        // SV identification problem
        //  (+) GLONASS NAV special case
        //      SV'X' is omitted 
        //  (+) faulty RINEX producer with unique constellation
        //      SV'X' is omitted
        Some(constellation::Constellation::Mixed) => sv::Sv::from_str(sv.trim())?,
        Some(c) => {
            let prn = u8::from_str_radix(sv.trim(), 10)?;  
            sv::Sv::new(c, prn)
        },
		_ => unreachable!(), // RINEX::NAV body while Type!=NAV
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
    Ok((
        epoch::Epoch::new(
            epoch::str2date(date)?,
            epoch::EpochFlag::default(),
        ),
        sv, 
        map))
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
