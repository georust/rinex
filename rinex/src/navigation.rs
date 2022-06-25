//! `NavigationData` parser and related methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use itertools::Itertools;
use serde_derive::Serialize;
use std::collections::{BTreeMap,HashMap};

use crate::sv;
use crate::epoch;
use crate::header;
use crate::version;
use crate::constellation;

include!(concat!(env!("OUT_DIR"),"/nav_data.rs"));

/// Record: NAV file content.
/// Data is encapsulated in as a `ComplexEnum`.   
/// Data is sorted by standardized identification code, contained in `navigation.json`,
/// for a given Satellite vehicule and for a given Epoch.
pub type Record = BTreeMap<epoch::Epoch, HashMap<sv::Sv, HashMap<String, ComplexEnum>>>;

/// `ComplexEnum` is record payload 
#[derive(Clone, PartialEq, Serialize, Debug)]
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

/// Builds `Record` entry for `NavigationData`
pub fn build_record_entry (header: &header::Header, content: &str)
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

    map.insert("msg".to_string(), msgtype);
    map.insert("type".to_string(), rectype);

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

    map.insert("ClockBias".to_string(), ComplexEnum::new("f32", svbias.trim())?); 
    map.insert("ClockDrift".to_string(), ComplexEnum::new("f32", svdrift.trim())?); 
    map.insert("ClockDriftRate".to_string(), ComplexEnum::new("f32", svdriftr.trim())?); 

    line = lines.next()
        .unwrap();

    let mut total: usize = 0; 
    let mut new_line = true;
    
    // NAV database for all following items
    // [1] identify revision for given satellite vehicule constellation
    //     (not using header.revision because it could be Constellation::Mixed
    let db_revision =  db_closest_revision(
        sv.constellation,
        header.version
    );
    if let Some(revision) = db_revision { // contained in db
        // [2] retrieve db items to parse
        let items : Vec<_> = NAV_MESSAGES
            .iter()
            .filter(|r| r.constellation == sv.constellation.to_3_letter_code())
            .map(|r| {
                r.revisions
                    .iter()
                    .filter(|r| // identified db revision
                        u8::from_str_radix(r.major,10).unwrap() == revision.major
                        && u8::from_str_radix(r.minor,10).unwrap() == revision.minor
                    )
                    .map(|r| &r.items)
                    .flatten()
            })
            .flatten()
            .collect();
        // parse items
        for item in items.iter() {
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
    Ok((
        epoch::Epoch::new(
            epoch::str2date(date)?,
            epoch::EpochFlag::default(), // always, for NAV
        ),
        sv,
        map,
    ))
}

/// Pushes observation record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    for epoch in record.keys() {
        match header.version.major {
            1|2 => write!(writer, " {} ",  epoch.date.format("%y %m %d %H %M %.6f").to_string())?,
              _ => write!(writer, "> {} ", epoch.date.format("%Y %m %d %H %M %.6f").to_string())?,
        }
    }
    Ok(())
}

/// Identifies closest revision contained in NAV database.   
/// Closest content is later used to identify data payload.    
/// Returns None if no database entries found for requested constellation or   
/// only newer revisions found for this constellation (older revisions are always prefered) 
fn db_closest_revision (constell: constellation::Constellation, desired_rev: version::Version) -> Option<version::Version> {
    let db = &NAV_MESSAGES;
    let revisions : Vec<_> = db.iter() // match requested constellation
        .filter(|rev| rev.constellation == constell.to_3_letter_code())
        .map(|rev| &rev.revisions)
        .flatten()
        .collect();
    if revisions.len() == 0 {
        return None // ---> constell not found in dB
    }
    let mut major_matches : Vec<_> = revisions.iter()
        .filter(|r| i8::from_str_radix(r.major,10).unwrap() - desired_rev.major as i8 == 0)
        .collect();
    if major_matches.len() > 0 {
        // --> desired_rev.major perfectly matched
        // --> try to match desired_rev.minor perfectly
        let minor_matches : Vec<_> = major_matches.iter()
            .filter(|r| i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8 == 0)
            .collect();
        if minor_matches.len() > 0 {
            // [+] .major perfectly matched
            // [+] .minor perfectly matched
            //     -> item is unique (if dB declaration is correct)
            //     -> return directly
            let major = u8::from_str_radix(minor_matches[0].major, 10).unwrap();
            let minor = u8::from_str_radix(minor_matches[0].minor, 10).unwrap();
            Some(version::Version::new(major, minor))
        } else {
            // [+] .major perfectly matched
            // [+] .minor not perfectly matched
            //    --> use closest older minor revision
            let mut to_sort : Vec<_> = major_matches
                .iter()
                .map(|r| (
                    u8::from_str_radix(r.major,10).unwrap(), // to later build object
                    u8::from_str_radix(r.minor,10).unwrap(), // to later build object
                    i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8 // for filter op
                )).collect();
            to_sort
                .sort_by(|a, b| b.2.cmp(&a.2)); // sort by delta value
            let to_sort : Vec<_> = to_sort
                .iter()
                .filter(|r| r.2 < 0) // retain negative deltas : only older revisions
                .collect();
            Some(version::Version::new(to_sort[0].0, to_sort[0].1))
        }
    } else {
        // ---> desired_rev.major not perfectly matched
        // ----> use closest older major revision
        let mut to_sort : Vec<_> = revisions
            .iter()
            .map(|r| (
                u8::from_str_radix(r.major,10).unwrap(), // to later build object
                i8::from_str_radix(r.major,10).unwrap() - desired_rev.major as i8, // for filter op
                u8::from_str_radix(r.minor,10).unwrap(), // to later build object
                i8::from_str_radix(r.minor,10).unwrap() - desired_rev.minor as i8, // for filter op
            )).collect(); 
        to_sort
            .sort_by(|a,b| b.1.cmp(&a.1)); // sort by major delta value
        let to_sort : Vec<_> = to_sort
            .iter()
            .filter(|r| r.1 < 0) // retain negative deltas only : only older revisions
            .collect();
        if to_sort.len() > 0 {
            // one last case:
            //   several minor revisions for given closest major revision
            //   --> prefer highest value
            let mut to_sort : Vec<_> = to_sort
                .iter()
                .duplicates_by(|r| r.1) // identical major deltas
                .collect();
            to_sort
                .sort_by(|a,b| b.3.cmp(&a.3)); // sort by minor deltas
            let last = to_sort.last().unwrap();
            Some(version::Version::new(last.0, last.2))
        } else {
            None // only newer revisions available, 
               // older revisions are always prefered
        }
    }
}

mod test {
    use std::str::FromStr;
    #[test]
    /// Tests static NAV database to be used in dedicated parser
    fn test_nav_database() {
        for n in super::NAV_MESSAGES.iter() { 
            super::constellation::Constellation::from_str(
                n.constellation
            ).unwrap();
            for r in n.revisions.iter() {
                u8::from_str_radix(r.major, 10).unwrap();
                u8::from_str_radix(r.minor, 10).unwrap();
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
                        super::ComplexEnum::new(v, &test).unwrap();
                    }
                }
            }
        }
    }
    use crate::version;
    use crate::constellation;
    use super::db_closest_revision;
    #[test]
    /// Tests closest revision identifier in db
    fn test_db_revision_finder() {
        let found = db_closest_revision(constellation::Constellation::Mixed, version::Version::default());
        assert_eq!(found, None); // Constellation::Mixed is not contained in db!
        // test GPS 1.0
        let target = version::Version::new(1, 0);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 4.0
        let target = version::Version::new(4, 0);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GPS 1.1 ==> 1.0
        let target = version::Version::new(1, 1);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.2 ==> 1.0
        let target = version::Version::new(1, 2);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.3 ==> 1.0
        let target = version::Version::new(1, 3);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = db_closest_revision(constellation::Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GLO 4.2 ==> 4.0
        let target = version::Version::new(4, 2);
        let found = db_closest_revision(constellation::Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GLO 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = db_closest_revision(constellation::Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test BDS 1.0 ==> does not exist 
        let target = version::Version::new(1, 0);
        let found = db_closest_revision(constellation::Constellation::Beidou, target); 
        assert_eq!(found, None); 
        // test BDS 1.4 ==> does not exist 
        let target = version::Version::new(1, 4);
        let found = db_closest_revision(constellation::Constellation::Beidou, target); 
        assert_eq!(found, None); 
        // test BDS 2.0 ==> does not exist 
        let target = version::Version::new(2, 0);
        let found = db_closest_revision(constellation::Constellation::Beidou, target); 
        assert_eq!(found, None); 
    }
}
