//! COSPAR ID number
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid COSPAR number")]
    InvalidFormat,
}

/// COSPAR ID number
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct COSPAR {
    /// Launch year
    year: u16,
    /// Launch number for that year, in chronological order.
    launch: u16,
    /// Up to three letter code representing the sequential
    /// identifier of a piece in a Launch.
    code: String,
}

impl std::fmt::Display for COSPAR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:04}-{:03}{}", self.year, self.launch, self.code)
    }
}

impl std::str::FromStr for COSPAR {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 9 {
            return Err(Error::InvalidFormat);
        }
        let offset = s.find('-').ok_or(Error::InvalidFormat)?;
        let (year, rem) = s.split_at(offset);
        let year = year.parse::<u16>().map_err(|_| Error::InvalidFormat)?;
        let launch = rem[1..4]
            .trim()
            .parse::<u16>()
            .map_err(|_| Error::InvalidFormat)?;
        Ok(Self {
            year,
            launch,
            code: rem[4..].trim().to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::cospar::COSPAR;
    use std::str::FromStr;
    #[test]
    fn cospar() {
        for (desc, expected) in [
            (
                "2018-080A",
                COSPAR {
                    year: 2018,
                    launch: 80,
                    code: "A".to_string(),
                },
            ),
            (
                "1996-068A",
                COSPAR {
                    year: 1996,
                    launch: 68,
                    code: "A".to_string(),
                },
            ),
        ] {
            let cospar = COSPAR::from_str(desc).unwrap();
            assert_eq!(cospar, expected);
            let recip = cospar.to_string();
            assert_eq!(recip, desc, "cospar reciprocal");
        }
    }
}
