//! sv.rs describes a satellite vehicule 
use thiserror::Error;
use std::str::FromStr;
use crate::constellation;

/// ̀`Sv` describes a Satellite Vehiculee
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Sv {
    /// prn identification # for this vehicule 
    pub prn: u8,
    /// `GNSS` Constellation to which this vehicule is tied to
    pub constellation: constellation::Constellation,
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

#[cfg(test)]
mod test {
    use super::*;
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
