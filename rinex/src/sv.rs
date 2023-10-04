//! Satellite vehicle
use super::{constellation, Constellation};
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// ̀`Sv` describes a Satellite Vehicle
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sv {
    /// PRN identification # for this vehicle
    pub prn: u8,
    /// `GNSS` Constellation to which this vehicle is tied to
    pub constellation: Constellation,
}

#[cfg(feature = "sbas")]
use hifitime::Epoch;

/*
 * On crate feature "sbas",
 * we have the ability to identify SBAS vehicles in detail.
 * The database is built by build.rs
 */
#[cfg(feature = "sbas")]
include!(concat!(env!("OUT_DIR"), "/sbas.rs"));

/// ̀`Sv` parsing & identification related errors
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParsingError {
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] constellation::ParsingError),
    #[error("sv prn# parsing error")]
    PRNParsing(#[from] std::num::ParseIntError),
}

impl Sv {
    /// Creates a new `Sv`
    pub fn new(constellation: Constellation, prn: u8) -> Self {
        Self { prn, constellation }
    }
    #[cfg(feature = "sbas")]
    /*
     * Tries to retrieve SBAS detailed definitions for self.
     * For that, we use the PRN number (+100 for SBAS)
     */
    pub(crate) fn sbas_definitions(&self) -> Option<&SBASHelper> {
        let to_find = (self.prn as u16) + 100;
        SBAS_VEHICLES
            .iter()
            .filter_map(|e| if e.prn == to_find { Some(e) } else { None })
            .reduce(|e, _| e)
    }
    #[cfg(feature = "sbas")]
    #[cfg_attr(docrs, doc(cfg(feature = "sbas")))]
    /// Returns datetime at which Self was either launched or its serviced was deployed.
    /// Datetime expressed as [Epoch] at midnight
    pub fn launched_date(&self) -> Option<Epoch> {
        let definition = self.sbas_definitions()?;
        Some(Epoch::from_gregorian_utc_at_midnight(
            definition.launched_year,
            definition.launched_month,
            definition.launched_day,
        ))
    }
}

impl std::str::FromStr for Sv {
    type Err = ParsingError;
    /*
     * Parse SV from "XYY" standardized format.
     * On "sbas" crate feature, we have the ability to identify
     * vehicles in detail. For example S23 is Eutelsat 5WB vehicle.
     */
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let constellation = Constellation::from_str(&string[0..1])?;
        let prn = u8::from_str_radix(&string[1..].trim(), 10)?;
        Ok(Sv { constellation, prn })
    }
}

#[cfg(not(feature = "sbas"))]
impl std::fmt::UpperHex for Sv {
    /*
     * Prints self as XYY standard format
     */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}", self)
    }
}

#[cfg(feature = "sbas")]
impl std::fmt::UpperHex for Sv {
    /*
     * Prints self as XYY standard format or possible SBAS determined identity
     */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(sbas) = self.sbas_definitions() {
            write!(f, "{}({})", sbas.constellation, sbas.id)
        } else {
            write!(f, "{:x}", self)
        }
    }
}

impl std::fmt::LowerHex for Sv {
    /*
     * Prints self as XYY standard format
     */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}{:02}", self.constellation, self.prn)
    }
}

impl std::fmt::Display for Sv {
    /*
     * Prints self as XYY standard format
     */
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:x}{:02}", self.constellation, self.prn)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn from_str() {
        for (descriptor, expected) in vec![
            ("G01", Sv::new(Constellation::GPS, 1)),
            ("G 1", Sv::new(Constellation::GPS, 1)),
            ("G33", Sv::new(Constellation::GPS, 33)),
            ("C01", Sv::new(Constellation::BeiDou, 1)),
            ("C 3", Sv::new(Constellation::BeiDou, 3)),
            ("R01", Sv::new(Constellation::Glonass, 1)),
            ("R 1", Sv::new(Constellation::Glonass, 1)),
            ("C254", Sv::new(Constellation::BeiDou, 254)),
            ("E4 ", Sv::new(Constellation::Galileo, 4)),
            ("R 9", Sv::new(Constellation::Glonass, 9)),
            ("I 3", Sv::new(Constellation::IRNSS, 3)),
            ("I16", Sv::new(Constellation::IRNSS, 16)),
            ("S22", Sv::new(Constellation::SBAS, 22)),
            ("S23", Sv::new(Constellation::SBAS, 23)),
            ("S25", Sv::new(Constellation::SBAS, 25)),
            ("S36", Sv::new(Constellation::SBAS, 36)),
        ] {
            let sv = Sv::from_str(descriptor);
            assert!(
                sv.is_ok(),
                "failed to parse sv from \"{}\" - {:?}",
                descriptor,
                sv.err().unwrap()
            );
            let sv = sv.unwrap();
            assert_eq!(
                sv, expected,
                "badly identified {} from \"{}\"",
                sv, descriptor
            );
        }
        /*
         * SBAS vehicles
         */
        for descriptor in vec!["S 5", "S 1", "S36", "S24"] {
            assert!(
                Sv::from_str(descriptor).is_ok(),
                "failed to parse SBAS from \"{}\"",
                descriptor
            );
        }
    }
    #[test]
    #[cfg(feature = "sbas")]
    fn upperhex() {
        for (id, expected) in vec![
            ("S00", "S00"), // unknown
            ("S01", "S01"), // unknown
            ("S23", "EGNOS(ASTRA-5B)"),
            ("S36", "EGNOS(SES-5)"),
        ] {
            let sv = Sv::from_str(id).unwrap();
            assert_eq!(format!("{:X}", sv), expected);
        }
    }
    #[test]
    #[cfg(feature = "sbas")]
    fn sbas_db_sanity() {
        for sbas in SBAS_VEHICLES.iter() {
            assert!(
                Constellation::from_str(sbas.constellation).is_ok(),
                "sbas database should only contain valid constellations: \"{}\"",
                sbas.constellation,
            );
            let _ = Epoch::from_gregorian_utc_at_midnight(
                sbas.launched_year,
                sbas.launched_month,
                sbas.launched_day,
            );
        }
    }
}
