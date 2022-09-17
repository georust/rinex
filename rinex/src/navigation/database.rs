//! NAV database for efficient navigation ephemeris
//! parsing, accross all revisions and constellations
use std::str::FromStr;
use itertools::Itertools;
use crate::version;
use thiserror::Error;
use crate::constellation::Constellation;

use super::{
	health,
};

include!(concat!(env!("OUT_DIR"),"/nav_db.rs"));

/// `DbItem` item is NAV record data base entry.
/// It serves as the actual Navigation record payload.
/// It is a complex data wrapper, for high level
/// record description, across all revisions and constellations 
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum DbItem {
    U8(u8),
    Str(String), 
    F32(f32),
    F64(f64),
	/// GPS/QZSS orbit/sv health indication
	Health(health::Health),
	/// GEO/SBAS orbit/sv health indication
	GeoHealth(health::GeoHealth),
	/// GAL orbit/sv health indication
	GalHealth(health::GalHealth),
	/// IRNSS orbit/sv health indication
	IrnssHealth(health::IrnssHealth),
}

//TODO: this method should produce a string in RINEX standards
// for easy data production
impl std::fmt::Display for DbItem {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DbItem::U8(u)  => write!(fmt, "{:X}", u),
            DbItem::Str(s) => write!(fmt, "{}", s),
            DbItem::F32(f) => write!(fmt, "{:.10e}", f),
            DbItem::F64(f) => write!(fmt, "{:.10e}", f),
			DbItem::Health(h) => write!(fmt, "{:?}", h),
			DbItem::GeoHealth(h) => write!(fmt, "{:?}", h),
			DbItem::GalHealth(h) => write!(fmt, "{:?}", h),
			DbItem::IrnssHealth(h) => write!(fmt, "{:?}", h),
        }
    }
}

/// `DbItem` related errors
#[derive(Error, Debug)]
pub enum DbItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
}

impl DbItem {
    /// Builds a `DbItem` from type descriptor and string content
    pub fn new (desc: &str, content: &str) -> Result<DbItem, DbItemError> {
        match desc {
            "str" => Ok(DbItem::Str(String::from(content))),
            "u8" => Ok(DbItem::U8(u8::from_str_radix(&content, 16)?)),
            "f32" => Ok(DbItem::F32(f32::from_str(&content.replace("D","e"))?)),
            "f64" => Ok(DbItem::F64(f64::from_str(&content.replace("D","e"))?)),
			_ => { // complex descriptor
				if desc.to_lowercase().contains("health") {
					let unsigned = u32::from_str_radix(content, 10)?;
					if desc.eq("health") {
						let flag = health::Health::from_bits(unsigned)
							.unwrap_or(health::Health::empty());
						Ok(DbItem::Health(flag))
					} else if desc.eq("galHealth") {
						let flag = health::GalHealth::from_bits(unsigned)
							.unwrap_or(health::GalHealth::empty());
						Ok(DbItem::GalHealth(flag))
					} else if desc.eq("geoHealth") { 
						let flag = health::GeoHealth::from_bits(unsigned)
							.unwrap_or(health::GeoHealth::empty());
						Ok(DbItem::GeoHealth(flag))
					} else if desc.eq("irnssHealth") {
						let flag = health::IrnssHealth::from_bits(unsigned)
							.unwrap_or(health::IrnssHealth::empty());
						Ok(DbItem::IrnssHealth(flag))
					} else {
            			Err(DbItemError::UnknownTypeDescriptor(desc.to_string()))
					}
				} else {
            		Err(DbItemError::UnknownTypeDescriptor(desc.to_string()))
				}
			},
        }
    }
	/// Unwraps DbItem as f32
    pub fn as_f32 (&self) -> Option<f32> {
        match self {
            DbItem::F32(f) => Some(f.clone()),
            _ => None,
        }
    }
	/// Unwraps DbItem as f64
    pub fn as_f64 (&self) -> Option<f64> {
        match self {
            DbItem::F64(f) => Some(f.clone()),
            _ => None,
        }
    }
	/// Unwraps DbItem as str
    pub fn as_str (&self) -> Option<String> {
        match self {
            DbItem::Str(s) => Some(s.clone()),
            _ => None,
        }
    }
	/// Unwraps DbItem as u8
    pub fn as_u8 (&self) -> Option<u8> {
        match self {
            DbItem::U8(u) => Some(u.clone()),
            _ => None,
        }
    }
	/// Unwraps Self as GPS/QZSS orbit Health indication 
	pub fn as_gps_health (&self) -> Option<health::Health> {
		match self {
			DbItem::Health(h) => Some(h.clone()),
			_ => None
		}
	}
	/// Unwraps Self as GEO/SBAS orbit Health indication 
	pub fn as_geo_health (&self) -> Option<health::GeoHealth> {
		match self {
			DbItem::GeoHealth(h) => Some(h.clone()),
			_ => None
		}
	}
	/// Unwraps Self as IRNSS orbit Health indication 
	pub fn as_irnss_health (&self) -> Option<health::IrnssHealth> {
		match self {
			DbItem::IrnssHealth(h) => Some(h.clone()),
			_ => None
		}
	}
}

/// Identifies closest (but older) revision contained in NAV database.   
/// Closest content (in time) is used during record parsing to identify and sort data.
/// Returns None if no database entries were found for requested constellation.
/// Returns None if only newer revisions were identified: we prefer older revisions.
pub fn closest_revision (constell: Constellation, desired_rev: version::Version) -> Option<version::Version> {
    let db = &NAV_MESSAGES;
    let revisions : Vec<_> = db.iter() // match requested constellation
        .filter(|rev| rev.constellation == constell.to_3_letter_code())
        .map(|rev| &rev.revisions)
        .flatten()
        .collect();
    if revisions.len() == 0 {
        return None // ---> constell not found in dB
    }
    let major_matches : Vec<_> = revisions.iter()
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

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_db_sanity() {
        for n in super::NAV_MESSAGES.iter() { 
            let c = Constellation::from_str(n.constellation);
            assert_eq!(c.is_ok(), true);
            for r in n.revisions.iter() {
                let major = u8::from_str_radix(r.major, 10);
                assert_eq!(major.is_ok(), true);
                let minor = u8::from_str_radix(r.minor, 10);
                assert_eq!(minor.is_ok(), true);
                for item in r.items.iter() {
                    let (k, v) = item;
                    if !k.contains(&"spare") {
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
                        let e = DbItem::new(v, &test);
                        assert_eq!(e.is_ok(), true);
                    }
                }
            }
        }
    }
    #[test]
    fn test_revision_finder() {
        let found = closest_revision(Constellation::Mixed, version::Version::default());
        assert_eq!(found, None); // Constellation::Mixed is not contained in db!
        // test GPS 1.0
        let target = version::Version::new(1, 0);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 4.0
        let target = version::Version::new(4, 0);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GPS 1.1 ==> 1.0
        let target = version::Version::new(1, 1);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.2 ==> 1.0
        let target = version::Version::new(1, 2);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.3 ==> 1.0
        let target = version::Version::new(1, 3);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GPS 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::GPS, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test GLO 4.2 ==> 4.0
        let target = version::Version::new(4, 2);
        let found = closest_revision(Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(4, 0)));
        // test GLO 1.4 ==> 1.0
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::Glonass, target); 
        assert_eq!(found, Some(version::Version::new(1, 0)));
        // test BDS 1.0 ==> does not exist 
        let target = version::Version::new(1, 0);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
        // test BDS 1.4 ==> does not exist 
        let target = version::Version::new(1, 4);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
        // test BDS 2.0 ==> does not exist 
        let target = version::Version::new(2, 0);
        let found = closest_revision(Constellation::BeiDou, target); 
        assert_eq!(found, None); 
    }
    #[test]
    fn test_db_item() {
        let e = DbItem::U8(10);
        assert_eq!(e.as_u8().is_some(), true);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_u8().unwrap();
        assert_eq!(u, 10);
        
        let e = DbItem::Str(String::from("Hello World"));
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_str().is_some(), true);
        let u = e.as_str().unwrap();
        assert_eq!(u, "Hello World");
        
        let e = DbItem::F32(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), true);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_f32().unwrap();
        assert_eq!(u, 10.0_f32);
        
        let e = DbItem::F64(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_f64().is_some(), true);
        assert_eq!(e.as_str().is_some(), false);
        let u = e.as_f64().unwrap();
        assert_eq!(u, 10.0_f64);
    }
}
