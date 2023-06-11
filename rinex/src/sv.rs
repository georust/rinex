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
pub enum Error {
    #[error("unknown constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse prn")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Sv {
    /// Creates a new `Sv`
    pub fn new(constellation: Constellation, prn: u8) -> Self {
        Self { prn, constellation }
    }
}

impl std::str::FromStr for Sv {
    type Err = Error;
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
    fn test_from_str() {
        let tests: Vec<&str> = vec!["C01", "C 3", "G33", "C254", "E4 ", "R 9"];
        for t in tests {
            assert!(Sv::from_str(t).is_ok());
        }
        // SBAS vehicles
        let sbas = Sv::from_str("S36");
        assert!(sbas.is_ok());
        assert_eq!(sbas.unwrap(), Sv::new(Constellation::Geo, 36));
        let sbas = Sv::from_str("S23");
        assert!(sbas.is_ok());
        assert_eq!(sbas.unwrap(), Sv::new(Constellation::Geo, 23));
    }
}
