//! `GNSS` constellations & associated methods
use thiserror::Error;
use serde_derive::{Deserialize, Serialize};

const GPS_STR_IDENTIFIER     : &str = "GPS";
const GLONASS_STR_IDENTIFIER : &str = "GLO"; 
const GALILEO_STR_IDENTIFIER : &str = "GAL"; 
const QZSS_STR_IDENTIFIER    : &str = "QZS";
const BEIDOU_STR_IDENTIFIER  : &str = "BDS";
const SBAS_STR_IDENTIFIER    : &str = "SBS";
const MIXED_STR_IDENTIFIER   : &str = "M";

/// Number of known ̀`GNSS` constellations
pub const CONSTELLATION_LENGTH: usize = 6;

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(Serialize, Deserialize)]
pub enum Constellation {
    /// `GPS` american constellation,
    GPS,
    /// `Glonass` russian constellation
    Glonass,
    /// `Beidou` chinese constellation
    Beidou,
    /// `QZSS` japanese constellation
    QZSS,
    /// `Galileo` european constellation
    Galileo,
    /// `Sbas` constellation
    Sbas,
    /// `Mixed` for Mixed constellations 
    /// RINEX files description
    Mixed,
}

impl Default for Constellation {
    /// Builds a default `GNSS::GPS` constellation
    fn default() -> Constellation {
        Constellation::GPS
    }
}

#[derive(Error, Debug)]
/// Constellation parsing & identification errors
pub enum ConstellationError {
    /// unable to identify constellation from str content
    #[error("unknown constellation code \"{0}\"")]
    UnknownConstellationCode(String),
}

impl Constellation {
    /// Builds a GNSS constellation from given
    /// three letter identiication code.
    /// This method is case insensitive
    pub fn from_3_letter_code (code: &str) -> Result<Constellation, ConstellationError> {
        if code.to_lowercase().eq("gps") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().eq("glo") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().eq("bds") {
            Ok(Constellation::Beidou)
        } else if code.to_lowercase().eq("gal") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().eq("qzs") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().eq("sbs") {
            Ok(Constellation::Sbas)
        } else {
            Err(ConstellationError::UnknownConstellationCode(code.to_string()))
        }
    }
    /// Builds a GNSS constellation from given
    /// one letter identiication code.
    /// This method is case insensitive 
    /// and discards all but first character in given code
    pub fn from_1_letter_code (code: &str) -> Result<Constellation, ConstellationError> {
        if code.to_lowercase().starts_with("g") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().starts_with("c") {
            Ok(Constellation::Beidou)
        } else if code.to_lowercase().starts_with("r") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().starts_with("j") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().starts_with("s") {
            Ok(Constellation::Sbas)
        } else if code.to_lowercase().starts_with("e") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().starts_with("m") {
            Ok(Constellation::Mixed)
        } else {
            Err(ConstellationError::UnknownConstellationCode(code.to_string()))
        }
    }
}

impl std::str::FromStr for Constellation {
    type Err = ConstellationError;
    /// Builds a GNSS Constellation from an str,
    /// either from a 3 letter code if strictly given 3 letters,
    /// or from total given content
    /// and finally trying to identify a 1 letter code.
    /// All xx letter code methods are case insensitive
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        if code.len() == 3 {
            Self::from_3_letter_code(code)
        } else if code.len() == 1 {
            Self::from_1_letter_code(code)
        } else {
            if code.to_lowercase().contains("gps") {
                Ok(Constellation::GPS)
            } else if code.to_lowercase().contains("glonass") {
                Ok(Constellation::Glonass)
            } else if code.to_lowercase().contains("galileo") {
                Ok(Constellation::Galileo)
            } else if code.to_lowercase().contains("qzss") {
                Ok(Constellation::QZSS)
            } else if code.to_lowercase().contains("beidou") {
                Ok(Constellation::Beidou)
            } else if code.to_lowercase().contains("sbas") {
                Ok(Constellation::Sbas)
            } else if code.to_lowercase().contains("mixed") {
                Ok(Constellation::Mixed)
            } else if code.to_lowercase().starts_with("m") {
                Ok(Constellation::Mixed)
            } else {
                Err(ConstellationError::UnknownConstellationCode(code.to_string()))
            }
        }
    }
}

impl std::fmt::Display for Constellation {
    /// formats a `GNSS` Constellation with single character identifier
    fn fmt (&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Constellation::GPS => fmt.write_str(GPS_STR_IDENTIFIER),
            Constellation::Glonass => fmt.write_str(GLONASS_STR_IDENTIFIER),
            Constellation::Galileo => fmt.write_str(GALILEO_STR_IDENTIFIER),
            Constellation::Beidou => fmt.write_str(BEIDOU_STR_IDENTIFIER),
            Constellation::QZSS => fmt.write_str(QZSS_STR_IDENTIFIER),
            Constellation::Sbas => fmt.write_str(SBAS_STR_IDENTIFIER),
            Constellation::Mixed => fmt.write_str(MIXED_STR_IDENTIFIER),
        }
    }
}
