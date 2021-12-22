use thiserror::Error;
use std::str::FromStr;

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
    Mixed, // mixed constellation records
}

impl Default for Constellation {
    fn default() -> Constellation {
        Constellation::GPS
    }
}

#[derive(Error, Debug)]
pub enum ConstellationError {
    #[error("unknown constellation \"{0}\"")]
    UnknownConstellation(String),
}

impl std::str::FromStr for Constellation {
    type Err = ConstellationError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("G") {
            Ok(Constellation::GPS)
        } else if s.starts_with("E") {
            Ok(Constellation::Galileo)
        } else if s.starts_with("R") {
            Ok(Constellation::Glonass)
        } else if s.starts_with("J") {
            Ok(Constellation::QZSS)
        } else if s.starts_with("C") {
            Ok(Constellation::Beidou)
        } else if s.starts_with("M") {
            Ok(Constellation::Mixed)
        } else {
            Err(ConstellationError::UnknownConstellation(s.to_string()))
        }
    }
}

impl std::fmt::Display for Constellation {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constellation::GPS => fmt.write_str("GPS"),
            Constellation::Glonass => fmt.write_str("GLO"),
            Constellation::Beidou => fmt.write_str("BDS"),
            Constellation::QZSS => fmt.write_str("QZS"),
            Constellation::Galileo => fmt.write_str("GAL"),
            _ => fmt.write_str("M"),
        }
    }
}
