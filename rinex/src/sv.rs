//! Satellite vehicule
use thiserror::Error;
use super::{constellation, Constellation};

#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// ̀`Sv` describes a Satellite Vehiculee
#[derive(Copy, Clone, Debug, Default)]
#[derive(PartialEq, Eq, Hash)]
#[derive(PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sv {
    /// PRN identification # for this vehicule 
    pub prn: u8,
    /// `GNSS` Constellation to which this vehicule is tied to
    pub constellation: Constellation,
}

/// ̀`Sv` parsing & identification related errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse prn")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Sv {
    /// Creates a new `Sv`
    pub fn new (constellation: Constellation, prn: u8) -> Self { 
        Self {
            prn, 
            constellation, 
        }
    }
}

impl std::str::FromStr for Sv {
    type Err = Error;
    /// Builds an `Sv` from XYY identification code.   
    /// code should strictly follow rinex conventions.   
    /// This method tolerates trailing whitespaces 
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        Ok(Sv {
            constellation: Constellation::from_1_letter_code(&s[0..1])?,
            prn: u8::from_str_radix(&s[1..].trim(), 10)?
        })
    }
}

impl std::fmt::Display for Sv {
    /// Formats self as XYY RINEX three letter code
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}{:02}", self.constellation.to_1_letter_code(), self.prn)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_sv_constructor() {
        let tests : Vec<&str> = vec![
            "C01", "C 3", "G33", "C254", "E4 ", "R 9",
        ];
        for t in tests {
            let _ = Sv::from_str(t).unwrap();
        }
    }
}
