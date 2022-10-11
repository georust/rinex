//! Satellite vehicule
use thiserror::Error;
use super::{constellation, Constellation};

#[cfg(feature = "serde")]
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserializer, Deserialize};

/// ̀`Sv` describes a Satellite Vehiculee
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, Hash)]
#[derive(PartialOrd, Ord)]
pub struct Sv {
    /// PRN identification # for this vehicule 
    pub prn: u8,
    /// `GNSS` Constellation to which this vehicule is tied to
    pub constellation: Constellation,
}

#[cfg(feature = "serde")]
impl Serialize for Sv {
    /// Dumps an `Sv` structure in RINEX standard format
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}{:02}", 
            self.constellation.to_1_letter_code(),
            self.prn);
        serializer.serialize_str(&s)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Sv {
    /// Builds an `Sv` structure from usual String description
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s)
            .map_err(serde::de::Error::custom)
    }
}

/// ̀`Sv` parsing & identification related errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("failed to parse prn")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Default for Sv {
    /// Builds a default `Sv`
    fn default() -> Sv {
        Sv {
            constellation: Constellation::default(),
            prn: 1
        }
    }
}

impl Sv {
    /// Creates a new `Sv` descriptor
    pub fn new (constellation: Constellation, prn: u8) -> Sv { Sv {constellation, prn }}
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
