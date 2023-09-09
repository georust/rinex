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
}

impl std::str::FromStr for Sv {
    type Err = ParsingError;
    /// Builds an `Sv` from XYY identification code.   
    /// code should strictly follow rinex conventions.   
    /// This method tolerates trailing whitespaces
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Sv {
            constellation: Constellation::from_1_letter_code(&s[0..1])?,
            prn: u8::from_str_radix(&s[1..].trim(), 10)?,
        })
    }
}

impl std::fmt::Display for Sv {
    /// Formats self as XYY RINEX three letter code
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            fmt,
            "{}{:02}",
            self.constellation.to_1_letter_code(),
            self.prn
        )
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
            ("S36", Sv::new(Constellation::Geo, 36)),
            ("S 6", Sv::new(Constellation::Geo, 6)),
        ] {
            let sv = Sv::from_str(descriptor);
            assert!(sv.is_ok(), "failed to parse sv from \"{}\"", descriptor);
            let sv = sv.unwrap();
            assert_eq!(
                sv, expected,
                "badly identified {} from \"{}\"",
                sv, descriptor
            );
        }
    }
}
