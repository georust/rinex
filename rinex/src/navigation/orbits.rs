//! NAV Orbits description, spanning all revisions
//! and constellations
use crate::version;
//use std::fmt::Display;
use super::health;
use crate::constellation::Constellation;
use bitflags::bitflags;
use itertools::Itertools;
use std::str::FromStr;
use thiserror::Error;

include!(concat!(env!("OUT_DIR"), "/nav_orbits.rs"));

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

bitflags! {
    #[derive(Default, Debug, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GloStatus: u32 {
        const GROUND_GPS_ONBOARD_OFFSET = 0x01;
        const ONBOARD_GPS_GROUND_OFFSET = 0x02;
        const ONBOARD_OFFSET = 0x03;
        const HALF_HOUR_VALIDITY = 0x04;
        const THREE_QUARTER_HOUR_VALIDITY = 0x06;
        const ONE_HOUR_VALIDITY = 0x07;
        const ODD_TIME_INTERVAL = 0x08;
        const SAT5_ALMANAC = 0x10;
        const DATA_UPDATED = 0x20;
        const MK = 0x40;
    }
}

/// `OrbitItem` item is Navigation ephemeris entry.
/// It is a complex data wrapper, for high level
/// record description, across all revisions and constellations
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum OrbitItem {
    /// unsigned byte
    U8(u8),
    /// signed byte
    I8(i8),
    /// single precision data
    F32(f32),
    /// double precision data
    F64(f64),
    /// GPS/QZSS orbit/sv health indication
    Health(health::Health),
    /// GLO orbit/sv health indication
    GloHealth(health::GloHealth),
    /// GLO NAV4 Orbit7 status mask
    GloStatus(GloStatus),
    /// GEO/SBAS orbit/sv health indication
    GeoHealth(health::GeoHealth),
    /// GAL orbit/sv health indication
    GalHealth(health::GalHealth),
    /// IRNSS orbit/sv health indication
    IrnssHealth(health::IrnssHealth),
}

/// `OrbitItem` related errors
#[derive(Error, Debug)]
pub enum OrbitItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
}

impl OrbitItem {
    /// Builds a `OrbitItem` from type descriptor and string content
    pub fn new(
        type_desc: &str,
        content: &str,
        constellation: Constellation,
    ) -> Result<OrbitItem, OrbitItemError> {
        match type_desc {
            "u8" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace("D", "e"))?;
                Ok(OrbitItem::U8(float as u8))
            },
            "i8" => {
                // float->signed conversion
                let float = f64::from_str(&content.replace("D", "e"))?;
                Ok(OrbitItem::I8(float as i8))
            },
            "f32" => Ok(OrbitItem::F32(f32::from_str(&content.replace("D", "e"))?)),
            "f64" => Ok(OrbitItem::F64(f64::from_str(&content.replace("D", "e"))?)),
            "gloStatus" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace("D", "e"))?;
                let unsigned = float as u32;
                let status = GloStatus::from_bits(unsigned).unwrap_or(GloStatus::empty());
                Ok(OrbitItem::GloStatus(status))
            },
            "health" => {
                // float->unsigned conversion
                let float = f64::from_str(&content.replace("D", "e"))?;
                let unsigned = float as u32;
                match constellation {
                    Constellation::GPS | Constellation::QZSS => {
                        let flag: health::Health = num::FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::Health::default());
                        Ok(OrbitItem::Health(flag))
                    },
                    Constellation::Glonass => {
                        let flag: health::GloHealth = num::FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::GloHealth::default());
                        Ok(OrbitItem::GloHealth(flag))
                    },
                    Constellation::Galileo => {
                        let flags = health::GalHealth::from_bits(unsigned as u8)
                            .unwrap_or(health::GalHealth::empty());
                        Ok(OrbitItem::GalHealth(flags))
                    },
                    Constellation::SBAS(_) | Constellation::Geo => {
                        let flag: health::GeoHealth = num::FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::GeoHealth::default());
                        Ok(OrbitItem::GeoHealth(flag))
                    },
                    Constellation::IRNSS => {
                        let flag: health::IrnssHealth = num::FromPrimitive::from_u32(unsigned)
                            .unwrap_or(health::IrnssHealth::default());
                        Ok(OrbitItem::IrnssHealth(flag))
                    },
                    _ => unreachable!(), // MIXED is not feasible here
                                         // as we use the current vehicle's constellation,
                                         // which is always defined
                }
            }, // "health"
            _ => Err(OrbitItemError::UnknownTypeDescriptor(type_desc.to_string())),
        }
    }
    /// Formats self following RINEX standards,
    /// mainly used when producing a file
    pub fn to_string(&self) -> String {
        match self {
            OrbitItem::U8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::I8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::F32(f) => format!("{:14.11E}", f),
            OrbitItem::F64(f) => format!("{:14.11E}", f),
            OrbitItem::Health(h) => format!("{:14.11E}", h),
            OrbitItem::GloHealth(h) => format!("{:14.11E}", h),
            OrbitItem::GeoHealth(h) => format!("{:14.11E}", h),
            OrbitItem::IrnssHealth(h) => format!("{:14.11E}", h),
            OrbitItem::GalHealth(h) => format!("{:14.11E}", h.bits() as f64),
            OrbitItem::GloStatus(h) => format!("{:14.11E}", h.bits() as f64),
        }
    }
    /// Unwraps OrbitItem as f32
    pub fn as_f32(&self) -> Option<f32> {
        match self {
            OrbitItem::F32(f) => Some(f.clone()),
            _ => None,
        }
    }
    /// Unwraps OrbitItem as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            OrbitItem::F64(f) => Some(f.clone()),
            _ => None,
        }
    }
    /// Unwraps OrbitItem as u8
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            OrbitItem::U8(u) => Some(u.clone()),
            _ => None,
        }
    }
    /// Unwraps OrbitItem as i8
    pub fn as_i8(&self) -> Option<i8> {
        match self {
            OrbitItem::I8(i) => Some(i.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GPS/QZSS orbit Health indication
    pub fn as_gps_health(&self) -> Option<health::Health> {
        match self {
            OrbitItem::Health(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GEO/SBAS orbit Health indication
    pub fn as_geo_health(&self) -> Option<health::GeoHealth> {
        match self {
            OrbitItem::GeoHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GLO orbit Health indication
    pub fn as_glo_health(&self) -> Option<health::GloHealth> {
        match self {
            OrbitItem::GloHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as GAL orbit Health indication
    pub fn as_gal_health(&self) -> Option<health::GalHealth> {
        match self {
            OrbitItem::GalHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
    /// Unwraps Self as IRNSS orbit Health indication
    pub fn as_irnss_health(&self) -> Option<health::IrnssHealth> {
        match self {
            OrbitItem::IrnssHealth(h) => Some(h.clone()),
            _ => None,
        }
    }
}

/// Identifies closest (but older) revision contained in NAV database.   
/// Closest content (in time) is used during record parsing to identify and sort data.
/// Returns None if no database entries were found for requested constellation.
/// Returns None if only newer revisions were identified: we prefer older revisions.
pub fn closest_revision(
    constell: Constellation,
    desired_rev: version::Version,
) -> Option<version::Version> {
    let db = &NAV_ORBITS;
    let revisions: Vec<_> = db
        .iter() // match requested constellation
        .filter(|rev| rev.constellation == constell.to_3_letter_code())
        .map(|rev| &rev.revisions)
        .flatten()
        .collect();
    if revisions.len() == 0 {
        return None; // ---> constell not found in dB
    }
    let major_matches: Vec<_> = revisions
        .iter()
        .filter(|r| i8::from_str_radix(r.major, 10).unwrap() - desired_rev.major as i8 == 0)
        .collect();
    if major_matches.len() > 0 {
        // --> desired_rev.major perfectly matched
        // --> try to match desired_rev.minor perfectly
        let minor_matches: Vec<_> = major_matches
            .iter()
            .filter(|r| i8::from_str_radix(r.minor, 10).unwrap() - desired_rev.minor as i8 == 0)
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
            let mut to_sort: Vec<_> = major_matches
                .iter()
                .map(|r| {
                    (
                        u8::from_str_radix(r.major, 10).unwrap(), // to later build object
                        u8::from_str_radix(r.minor, 10).unwrap(), // to later build object
                        i8::from_str_radix(r.minor, 10).unwrap() - desired_rev.minor as i8, // for filter op
                    )
                })
                .collect();
            to_sort.sort_by(|a, b| b.2.cmp(&a.2)); // sort by delta value
            let to_sort: Vec<_> = to_sort
                .iter()
                .filter(|r| r.2 < 0) // retain negative deltas : only older revisions
                .collect();
            Some(version::Version::new(to_sort[0].0, to_sort[0].1))
        }
    } else {
        // ---> desired_rev.major not perfectly matched
        // ----> use closest older major revision
        let mut to_sort: Vec<_> = revisions
            .iter()
            .map(|r| {
                (
                    u8::from_str_radix(r.major, 10).unwrap(), // to later build object
                    i8::from_str_radix(r.major, 10).unwrap() - desired_rev.major as i8, // for filter op
                    u8::from_str_radix(r.minor, 10).unwrap(), // to later build object
                    i8::from_str_radix(r.minor, 10).unwrap() - desired_rev.minor as i8, // for filter op
                )
            })
            .collect();
        to_sort.sort_by(|a, b| b.1.cmp(&a.1)); // sort by major delta value
        let to_sort: Vec<_> = to_sort
            .iter()
            .filter(|r| r.1 < 0) // retain negative deltas only : only older revisions
            .collect();
        if to_sort.len() > 0 {
            // one last case:
            //   several minor revisions for given closest major revision
            //   --> prefer highest value
            let mut to_sort: Vec<_> = to_sort
                .iter()
                .duplicates_by(|r| r.1) // identical major deltas
                .collect();
            to_sort.sort_by(|a, b| b.3.cmp(&a.3)); // sort by minor deltas
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
    fn test_db_orbits_sanity() {
        for n in NAV_ORBITS.iter() {
            let c = Constellation::from_str(n.constellation);
            assert_eq!(c.is_ok(), true);
            let c = c.unwrap();
            for r in n.revisions.iter() {
                let major = u8::from_str_radix(r.major, 10);
                assert_eq!(major.is_ok(), true);
                let minor = u8::from_str_radix(r.minor, 10);
                assert_eq!(minor.is_ok(), true);
                for item in r.items.iter() {
                    let (k, v) = item;
                    if !k.contains(&"spare") {
                        let test = String::from("0.000E0"); // testdata
                        let e = OrbitItem::new(v, &test, c);
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
        let e = OrbitItem::U8(10);
        assert_eq!(e.as_u8().is_some(), true);
        assert_eq!(e.as_f32().is_some(), false);
        let u = e.as_u8().unwrap();
        assert_eq!(u, 10);

        let e = OrbitItem::F32(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), true);
        let u = e.as_f32().unwrap();
        assert_eq!(u, 10.0_f32);

        let e = OrbitItem::F64(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_f32().is_some(), false);
        assert_eq!(e.as_f64().is_some(), true);
        let u = e.as_f64().unwrap();
        assert_eq!(u, 10.0_f64);
    }
}
