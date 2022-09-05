//! Satellite vehicule representation 
use thiserror::Error;
use crate::constellation;

#[cfg(feature = "with-serde")]
use std::str::FromStr;
use serde::{Serialize, Serializer, Deserializer, Deserialize};

/// ̀`Sv` describes a Satellite Vehiculee
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, Hash)]
pub struct Sv {
    /// prn identification # for this vehicule 
    pub prn: u8,
    /// `GNSS` Constellation to which this vehicule is tied to
    pub constellation: constellation::Constellation,
}

impl std::cmp::PartialOrd for Sv {
    fn partial_cmp (&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        let (c1, c2) = (self.constellation, rhs.constellation); 
        if c1 == c2 { // same constellation: PRN # differientiates
            self.prn.partial_cmp(&rhs.prn)
        } else { // By alphabetical order
            c1.to_1_letter_code().partial_cmp(c2.to_1_letter_code())
        }
    }
}

impl std::cmp::Ord for Sv {
    fn cmp (&self, rhs: &Self) -> std::cmp::Ordering {
        let (c1, c2) = (self.constellation, rhs.constellation); 
        if c1 == c2 { // same constellation: PRN # differentiates
            self.prn.cmp(&rhs.prn)
        } else { // By alphabetical order
            c1.to_1_letter_code().cmp(c2.to_1_letter_code())
        }
    }
}

#[cfg(feature = "with-serde")]
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

#[cfg(feature = "with-serde")]
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
            constellation: constellation::Constellation::default(),
            prn: 1
        }
    }
}

impl Sv {
    /// Creates a new `Sv` descriptor
    pub fn new (constellation: constellation::Constellation, prn: u8) -> Sv { Sv {constellation, prn }}
}

impl std::str::FromStr for Sv {
    type Err = Error;
    /// Builds an `Sv` from XYY identification code.   
    /// code should strictly follow rinex conventions.   
    /// This method tolerates trailing whitespaces 
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        Ok(Sv {
            constellation: constellation::Constellation::from_1_letter_code(&s[0..1])?,
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
