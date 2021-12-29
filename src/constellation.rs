use thiserror::Error;

const GPS_CHAR_IDENTIFIER     : char = 'G';
const GLONASS_CHAR_IDENTIFIER : char = 'R'; 
const GALILEO_CHAR_IDENTIFIER : char = 'E'; 
const QZSS_CHAR_IDENTIFIER    : char = 'J';
const BEIDOU_CHAR_IDENTIFIER  : char = 'C';
const SBAS_CHAR_IDENTIFIER    : char = 'S';
const MIXED_CHAR_IDENTIFIER   : char = 'M';

const GPS_STR_IDENTIFIER     : &str = "GPS";
const GLONASS_STR_IDENTIFIER : &str = "GLO"; 
const GALILEO_STR_IDENTIFIER : &str = "GAL"; 
const QZSS_STR_IDENTIFIER    : &str = "QZS";
const BEIDOU_STR_IDENTIFIER  : &str = "BDS";
const SBAS_STR_IDENTIFIER    : &str = "SBS";
const MIXED_STR_IDENTIFIER   : &str = "M";

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

impl Constellation {
    pub fn from_char (c: char) -> Result<Constellation, ConstellationError> {
        match c {
            GPS_CHAR_IDENTIFIER => Ok(Constellation::GPS),
            GLONASS_CHAR_IDENTIFIER => Ok(Constellation::Glonass),
            GALILEO_CHAR_IDENTIFIER => Ok(Constellation::Galileo),
            QZSS_CHAR_IDENTIFIER => Ok(Constellation::QZSS),
            BEIDOU_CHAR_IDENTIFIER => Ok(Constellation::Beidou),
            MIXED_CHAR_IDENTIFIER => Ok(Constellation::Mixed),
            SBAS_CHAR_IDENTIFIER => Ok(Constellation::Sbas),
            _ => Err(ConstellationError::UnknownConstellation(c.to_string())),
        }
    }
}

impl std::str::FromStr for Constellation {
    type Err = ConstellationError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.contains(GPS_STR_IDENTIFIER) {
            Ok(Constellation::GPS)
        } else if s.contains(GLONASS_STR_IDENTIFIER) {
            Ok(Constellation::Glonass)
        } else if s.contains(GALILEO_STR_IDENTIFIER) {
            Ok(Constellation::Galileo)
        } else if s.contains(QZSS_STR_IDENTIFIER) {
            Ok(Constellation::QZSS)
        } else if s.contains(BEIDOU_STR_IDENTIFIER) {
            Ok(Constellation::Beidou)
        } else if s.contains(SBAS_STR_IDENTIFIER) {
            Ok(Constellation::Sbas)
        } else if s.contains(MIXED_STR_IDENTIFIER) {
            Ok(Constellation::Mixed)
        } else {
            match s.chars().nth(0)
                .unwrap() {
                GPS_CHAR_IDENTIFIER => Ok(Constellation::GPS),
                GLONASS_CHAR_IDENTIFIER => Ok(Constellation::Glonass),
                GALILEO_CHAR_IDENTIFIER => Ok(Constellation::Galileo),
                QZSS_CHAR_IDENTIFIER => Ok(Constellation::QZSS),
                BEIDOU_CHAR_IDENTIFIER => Ok(Constellation::Beidou),
                MIXED_CHAR_IDENTIFIER => Ok(Constellation::Mixed),
                SBAS_CHAR_IDENTIFIER => Ok(Constellation::Sbas),
                _ => Err(ConstellationError::UnknownConstellation(s.to_string())),
            }
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
