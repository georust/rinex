//! NAV Orbits description, spanning all revisions and constellations
use crate::version;
use bitflags::bitflags;
use std::str::FromStr;
use thiserror::Error;

use crate::navigation::health::GPSHealth;
use crate::navigation::Health;

include!(concat!(env!("OUT_DIR"), "/nav_orbits.rs"));

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
    /// unsigned 32 bit value
    U32(u32),
    /// double precision data
    F64(f64),
    /// Sv Health indication
    Health(Health),
    /// GLO NAV4 Orbit7 status mask
    GloStatus(GloStatus),
}

impl From<u32> for OrbitItem {
    fn from(value: u32) -> Self {
        Self::U32(value)
    }
}

impl From<f64> for OrbitItem {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
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
            "u32" => {
                // float->signed conversion
                let float = f64::from_str(&content.replace("D", "e"))?;
                Ok(OrbitItem::U32(float as u32))
            },
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
                let sv_health = match constellation {
                    Constellation::GPS => {
                        let gps = GPSHealth::default();
                        Health::from(gps)
                    },
                    _ => Health::default(),
                };
                Ok(OrbitItem::Health(sv_health))
            },
            _ => Err(OrbitItemError::UnknownTypeDescriptor(type_desc.to_string())),
        }
    }
    /// Formats self following RINEX standards,
    /// mainly used when producing a file
    pub fn to_string(&self) -> String {
        match self {
            OrbitItem::U8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::I8(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::U32(n) => format!("{:14.11E}", *n as f64),
            OrbitItem::F64(f) => format!("{:14.11E}", f),
            OrbitItem::Health(h) => format!("{:14.11E}", h),
            OrbitItem::GloStatus(h) => format!("{:14.11E}", h.bits() as f64),
        }
    }
    /// Unwraps OrbitItem as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            OrbitItem::F64(f) => Some(f.clone()),
            _ => None,
        }
    }
    /// Unwraps self as u32 (if possible)
    pub fn as_u32(&self) -> Option<u32> {
        match self {
            OrbitItem::U32(v) => Some(*v),
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
    /// Unwraps Self as [`Health`]
    pub fn as_health(&self) -> Option<Health> {
        match self {
            OrbitItem::Health(h) => Some(h.clone()),
            _ => None,
        }
    }
}

/*
 * Identifies closest (but older) revision contained in NAV database.
 * Closest content (in time) is used during record parsing to identify and sort data.
 * Returns None
 *   - if no database entries were found for requested constellation.
 *   - or only newer revision exist : we prefer matching on older revisions
 */
pub(crate) fn closest_nav_standards(
    constellation: Constellation,
    revision: version::Version,
    msg: NavMsgType,
) -> Option<&'static NavHelper<'static>> {
    let database = &NAV_ORBITS;
    // start by trying to locate desired revision.
    // On each mismatch, we decrement and move on to next major/minor combination.
    let (mut major, mut minor): (u8, u8) = revision.into();
    loop {
        // filter on both:
        //  + Exact Constellation
        //  + Exact NavMsgType
        //  + Exact revision we're currently trying to locate
        //    algorithm: decreasing, starting from desired revision
        let items: Vec<_> = database
            .iter()
            .filter(|item| {
                item.constellation == constellation
                    && item.msg == msg
                    && item.version == Version::new(major, minor)
            })
            .collect();

        if items.len() == 0 {
            if minor == 0 {
                // we're done with this major
                // -> downgrade to previous major
                //    we start @ minor = 10, which is
                //    larger than most revisions we know
                if major == 0 {
                    // we're done: browsed all possibilities
                    break;
                } else {
                    major -= 1;
                    minor = 10;
                }
            } else {
                minor -= 1;
            }
        } else {
            return Some(items[0]);
        }
    } // loop
    None
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn orbit_database_sanity() {
        for frame in NAV_ORBITS.iter() {
            // Test data fields description
            let constellation = frame.constellation;
            for (key, value) in frame.items.iter() {
                let fake_content: Option<String> = match value {
                    &"f64" => Some(String::from("0.000")), // like we would parse it,
                    &"u32" => Some(String::from("0.000")),
                    &"u8" => Some(String::from("0.000")),
                    &"spare" => None, // such fields are actually dropped
                    _ => None,
                };
                if let Some(content) = fake_content {
                    // Item construction, based on this descriptor, must work.
                    // Like we use it when parsing..
                    let e = OrbitItem::new(value, &content, constellation);
                    assert!(
                        e.is_ok() == true,
                        "failed to build Orbit Item from (\"{}\", \"{}\", \"{}\")",
                        key,
                        value,
                        constellation,
                    );
                }
            }
        }
    }
    #[test]
    fn nav_standards_finder() {
        // Constellation::Mixed is not contained in db!
        assert_eq!(
            closest_nav_standards(Constellation::Mixed, Version::default(), NavMsgType::LNAV),
            None,
            "Mixed GNSS constellation is or should not exist in the DB"
        );

        // Test existing (exact match) entries
        for (constellation, rev, msg) in vec![
            (Constellation::GPS, Version::new(1, 0), NavMsgType::LNAV),
            (Constellation::GPS, Version::new(2, 0), NavMsgType::LNAV),
            (Constellation::GPS, Version::new(4, 0), NavMsgType::LNAV),
            (Constellation::GPS, Version::new(4, 0), NavMsgType::CNAV),
            (Constellation::GPS, Version::new(4, 0), NavMsgType::CNV2),
            (Constellation::Glonass, Version::new(2, 0), NavMsgType::LNAV),
            (Constellation::Glonass, Version::new(3, 0), NavMsgType::LNAV),
            (Constellation::Galileo, Version::new(3, 0), NavMsgType::LNAV),
            (Constellation::Galileo, Version::new(4, 0), NavMsgType::INAV),
            (Constellation::Galileo, Version::new(4, 0), NavMsgType::FNAV),
            (Constellation::QZSS, Version::new(3, 0), NavMsgType::LNAV),
            (Constellation::QZSS, Version::new(4, 0), NavMsgType::LNAV),
            (Constellation::QZSS, Version::new(4, 0), NavMsgType::CNAV),
            (Constellation::QZSS, Version::new(4, 0), NavMsgType::CNV2),
            (Constellation::BeiDou, Version::new(3, 0), NavMsgType::LNAV),
            (Constellation::BeiDou, Version::new(4, 0), NavMsgType::D1),
            (Constellation::BeiDou, Version::new(4, 0), NavMsgType::D2),
            (Constellation::BeiDou, Version::new(4, 0), NavMsgType::CNV1),
            (Constellation::BeiDou, Version::new(4, 0), NavMsgType::CNV2),
            (Constellation::BeiDou, Version::new(4, 0), NavMsgType::CNV3),
            (Constellation::Geo, Version::new(4, 0), NavMsgType::SBAS),
        ] {
            let found = closest_nav_standards(constellation, rev, msg);
            assert!(
                found.is_some(),
                "should have identified {}:V{} ({}) frame that actually exists in DB",
                constellation,
                rev,
                msg
            );

            let standards = found.unwrap();
            assert!(
                standards.constellation == constellation,
                "bad constellation identified \"{}\", expecting \"{}\"",
                constellation,
                standards.constellation
            );
            assert!(
                standards.version == rev,
                "should have matched {} V{} exactly, because it exists in DB",
                constellation,
                rev,
            );
        }

        // Test cases where the nearest revision is used, not that exact revision
        for (constellation, desired, expected, msg) in vec![
            (
                Constellation::GPS,
                Version::new(5, 0),
                Version::new(4, 0),
                NavMsgType::LNAV,
            ),
            (
                Constellation::GPS,
                Version::new(4, 1),
                Version::new(4, 0),
                NavMsgType::LNAV,
            ),
            (
                Constellation::Glonass,
                Version::new(3, 4),
                Version::new(3, 0),
                NavMsgType::LNAV,
            ),
        ] {
            let found = closest_nav_standards(constellation, desired, msg);
            assert!(
                found.is_some(),
                "should have converged for \"{}\" V\"{}\" (\"{}\") to nearest frame revision",
                constellation,
                desired,
                msg
            );

            let standards = found.unwrap();

            assert!(
                standards.constellation == constellation,
                "bad constellation identified \"{}\", expecting \"{}\"",
                constellation,
                standards.constellation
            );

            assert!(
                standards.version == expected,
                "closest_nav_standards() converged to wrong revision {}:{}({}) while \"{}\" was expected", 
                constellation,
                desired,
                msg,
                expected);
        }
    }
    #[test]
    fn test_db_item() {
        let e = OrbitItem::U8(10);
        assert_eq!(e.as_u8().is_some(), true);
        assert_eq!(e.as_u32().is_some(), false);
        let u = e.as_u8().unwrap();
        assert_eq!(u, 10);

        let e = OrbitItem::F64(10.0);
        assert_eq!(e.as_u8().is_some(), false);
        assert_eq!(e.as_u32().is_some(), false);
        assert_eq!(e.as_f64().is_some(), true);
        let u = e.as_f64().unwrap();
        assert_eq!(u, 10.0_f64);

        let e = OrbitItem::U32(1);
        assert_eq!(e.as_u32().is_some(), true);
        assert_eq!(e.as_f64().is_some(), false);
        let u = e.as_u32().unwrap();
        assert_eq!(u, 1_u32);
    }
}
