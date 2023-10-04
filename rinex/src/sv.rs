//! Satellite vehicle
use thiserror::Error;
use super::{constellation, Constellation};

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

#[cfg(feature = "sbas")]
use std::str::FromStr;

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
    fn sbas_definitions(&self) -> Option<&SBASHelper> {
        let to_find = (self.prn as u16) +100;
        SBAS_VEHICLES.iter()
            .filter_map(|e| {
                if e.prn == to_find {
                    Some(e)
                } else {
                    None
                }
            })
            .reduce(|e, _| e)
    }
    #[cfg(feature = "sbas")]
    // Tries to retrieve SBAS detailed ID, from the database
    pub(crate) fn sbas_identity(&self) -> Option<String> {
        let definition = self.sbas_definitions()?;
        Some(definition.id.to_string())
    }
    #[cfg_attr(docrs, doc(cfg(feature = "sbas")))]
    /// Returns datetime at which Self was either launched or its serviced was deployed.
    /// Datetime expressed as [Epoch] at midnight
    pub fn launched_date(&self) -> Option<Epoch> {
        let definition = self.sbas_definitions()?;
        Some(Epoch::default())
        //pub fn from_gregorian_utc_at_midnight(year: i32, month: u8, day: u8) -> Self {
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
        let content = string.trim();
        let constellation = Constellation::from_str(&content[0..1])?;
        let prn = u8::from_str_radix(&content[1..].trim(), 10)?;

        if cfg!(feature = "sbas") && constellation == Constellation::SBAS {
            // identify in detail, using prebuilt database
            let found = &SBAS_VEHICLES.iter()
                .filter_map(|e| {
                    let sbas_prn = prn as u16 +100;
                    if e.prn == sbas_prn { 
                        Some((e.constellation, sbas_prn))
                    } else {
                        None
                    }
                })
                .reduce(|e, _| e);
            if let Some((constell_str, _)) = found {
                // this can't fail: database only contains valid description
                let constellation = Constellation::from_str(constell_str)
                    .unwrap();
                Ok(Sv {
                    constellation,
                    prn,
                })
            } else {
                Ok(Sv {
                    constellation,
                    prn,
                })
            }
        } else {
            Ok(Sv {
                constellation,
                prn,
            })
        }
    }
}

impl std::fmt::Display for Sv {
    /// Formats self as XYY RINEX three letter code
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.constellation.is_sbas() {
            /*
             * in case we have the "sbas" feature, print the real vehicule identity 
             */
            if cfg!(feature = "sbas") {
                if let Some(id) = self.sbas_identity() {
                    write!(fmt, "{}", id)
                } else {
                    write!(fmt, "{:x}{:02}", self.constellation, self.prn)
                }
            } else {
                write!(fmt, "{:x}{:02}", self.constellation, self.prn)
            }
        } else {
            write!(fmt, "{:x}{:02}", self.constellation, self.prn)
        }
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
        ] {
            let sv = Sv::from_str(descriptor);
            assert!(sv.is_ok(), "failed to parse sv from \"{}\" - {:?}", descriptor, sv.err().unwrap());
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
        for descriptor in vec![
            "S 5", "S 1", "S36", "S24"
        ] {
            assert!(Sv::from_str(descriptor).is_ok(), "failed to parse SBAS from \"{}\"", descriptor);
        }
    }
    #[cfg(feature = "sbas")]
    #[test]
    fn sbas_id() {
        for (id, expected) in vec![
            ("S20", "S20"), // unknown
            ("S23", "ASTRA 5B"),
        ] {
            let sv = Sv::from_str(id)
                .unwrap();
            assert_eq!(sv.to_string(), expected);
        }
    }
    #[cfg(feature = "sbas")]
    #[test]
    fn sbas_db_sanity() {
        for sbas in SBAS_VEHICLES.iter() {
            assert!(Constellation::from_str(sbas.constellation).is_ok(),
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
