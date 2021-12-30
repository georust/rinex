use thiserror::Error;

const GPS_STR_IDENTIFIER     : &str = "GPS";
const GLONASS_STR_IDENTIFIER : &str = "GLO"; 
const GALILEO_STR_IDENTIFIER : &str = "GAL"; 
const QZSS_STR_IDENTIFIER    : &str = "QZS";
const BEIDOU_STR_IDENTIFIER  : &str = "BDS";
const SBAS_STR_IDENTIFIER    : &str = "SBS";
const MIXED_STR_IDENTIFIER   : &str = "M";

/// Number of known ̀`GNSS` constellations
pub const CONSTELLATION_NUMBER: u8 = 6;

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
    Sbas,
    Mixed,
}

impl Default for Constellation {
    /// Builds a default `GNSS` constellation
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
        if s.to_lowercase().contains("gps") {
            Ok(Constellation::GPS)
        } else if s.to_lowercase().contains("glonass") {
            Ok(Constellation::Glonass)
        } else if s.to_lowercase().contains("galileo") {
            Ok(Constellation::Galileo)
        } else if s.to_lowercase().contains("qzss") {
            Ok(Constellation::QZSS)
        } else if s.to_lowercase().contains("beidou") {
            Ok(Constellation::Beidou)
        } else if s.to_lowercase().contains("mixed") {
            Ok(Constellation::Mixed)
        } else if s.to_lowercase().starts_with("m") {
            Ok(Constellation::Mixed)
        } else {
            Err(ConstellationError::UnknownConstellation(s.to_string()))
        }
    }
}

impl std::fmt::Display for Constellation {
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constellation::GPS => fmt.write_str(GPS_STR_IDENTIFIER),
            Constellation::Glonass => fmt.write_str(GLONASS_STR_IDENTIFIER),
            Constellation::Galileo => fmt.write_str(GALILEO_STR_IDENTIFIER),
            Constellation::Beidou => fmt.write_str(BEIDOU_STR_IDENTIFIER),
            Constellation::QZSS => fmt.write_str(QZSS_STR_IDENTIFIER),
            Constellation::Sbas => fmt.write_str(SBAS_STR_IDENTIFIER),
            Constellation::Mixed => fmt.write_str(MIXED_STR_IDENTIFIER),
            _ => fmt.write_str("Unknown"),
        }
    }
}
