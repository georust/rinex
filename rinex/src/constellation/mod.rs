//! `GNSS` constellations & associated methods
use hifitime::TimeScale;
use thiserror::Error;

mod augmentation;
pub use augmentation::Augmentation;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "sbas")]
pub use augmentation::sbas_selection_helper;

/// Constellation parsing & identification related errors
#[derive(Error, Clone, Debug, PartialEq)]
pub enum ParsingError {
    #[error("unknown constellation \"{0}\"")]
    Unknown(String),
    #[error("unrecognized constellation \"{0}\"")]
    Unrecognized(String),
    #[error("unknown constellation format \"{0}\"")]
    Format(String),
}

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Constellation {
    /// `GPS` american constellation,
    #[default]
    GPS,
    /// `Glonass` russian constellation
    Glonass,
    /// `BeiDou` chinese constellation
    BeiDou,
    /// `QZSS` japanese constellation
    QZSS,
    /// `Galileo` european constellation
    Galileo,
    /// `Geo` : stationnary satellite,
    /// also serves as SBAS with unknown augmentation system
    Geo,
    /// `SBAS`
    SBAS(Augmentation),
    /// `IRNSS` constellation,
    /// now officially renamed "NavIC"
    IRNSS,
    /// `Mixed` for Mixed constellations
    /// RINEX files description
    Mixed,
}

impl std::fmt::Display for Constellation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.to_3_letter_code())
    }
}

impl Constellation {
    /*
     * Identifies GNSS constellation from standard 1 letter code
     * but can insensitive.
     * Mostly used in Self::from_str (public method)
     */
    pub(crate) fn from_1_letter_code(code: &str) -> Result<Self, ParsingError> {
        if code.len() != 1 {
            return Err(ParsingError::Format(code.to_string()));
        }

        let lower = code.to_lowercase();
        if lower.eq("g") {
            Ok(Self::GPS)
        } else if lower.eq("r") {
            Ok(Self::Glonass)
        } else if lower.eq("c") {
            Ok(Self::BeiDou)
        } else if lower.eq("e") {
            Ok(Self::Galileo)
        } else if lower.eq("j") {
            Ok(Self::QZSS)
        } else if lower.eq("s") {
            Ok(Self::Geo)
        } else if lower.eq("i") {
            Ok(Self::IRNSS)
        } else if lower.eq("m") {
            Ok(Self::Mixed)
        } else {
            Err(ParsingError::Unknown(code.to_string()))
        }
    }
    /*
     * Identifies Constellation from stanadrd 3 letter code, case insensitive.
     * Used in public Self::from_str, or some place else in that crate.
     */
    pub(crate) fn from_3_letter_code(code: &str) -> Result<Self, ParsingError> {
        if code.len() != 3 {
            return Err(ParsingError::Format(code.to_string()));
        }

        let lower = code.to_lowercase();
        if lower.eq("gps") {
            Ok(Self::GPS)
        } else if lower.eq("glo") {
            Ok(Self::Glonass)
        } else if lower.eq("bds") {
            Ok(Self::BeiDou)
        } else if lower.eq("gal") {
            Ok(Self::Galileo)
        } else if lower.eq("qzs") {
            Ok(Self::QZSS)
        } else if lower.eq("sbs") | lower.eq("geo") {
            Ok(Self::Geo)
        } else if lower.eq("irn") {
            Ok(Self::IRNSS)
        } else {
            Err(ParsingError::Unknown(code.to_string()))
        }
    }
    /*
     * Identifies `gnss` constellation from given standard plain name,
     * like "GPS", or "Galileo". This method is not case sensitive.
     * Used in public Self::from_str, or some place else in that crate.
     */
    pub(crate) fn from_plain_name(code: &str) -> Result<Constellation, ParsingError> {
        let lower = code.to_lowercase();
        if lower.contains("gps") {
            Ok(Self::GPS)
        } else if lower.contains("glonass") {
            Ok(Self::Glonass)
        } else if lower.contains("galileo") {
            Ok(Self::Galileo)
        } else if lower.contains("qzss") {
            Ok(Self::QZSS)
        } else if lower.contains("beidou") {
            Ok(Self::BeiDou)
        } else if lower.contains("sbas") {
            Ok(Self::Geo)
        } else if lower.contains("geo") {
            Ok(Self::Geo)
        } else if lower.contains("irnss") {
            Ok(Self::IRNSS)
        } else if lower.contains("mixed") {
            Ok(Self::Mixed)
        } else {
            Err(ParsingError::Unrecognized(code.to_string()))
        }
    }
    /// Converts self into time scale
    pub fn to_timescale(&self) -> Option<TimeScale> {
        match self {
            Self::GPS | Self::QZSS => Some(TimeScale::GPST),
            Self::Galileo => Some(TimeScale::GST),
            Self::BeiDou => Some(TimeScale::BDT),
            Self::Geo | Self::SBAS(_) => Some(TimeScale::GPST),
            // this is wrong but we can't do better
            Self::Glonass | Self::IRNSS => Some(TimeScale::UTC),
            _ => None,
        }
    }
    /// Converts self to 1 letter code (RINEX standard code)
    pub(crate) fn to_1_letter_code(&self) -> &str {
        match self {
            Self::GPS => "G",
            Self::Glonass => "R",
            Self::Galileo => "E",
            Self::BeiDou => "C",
            Self::SBAS(_) | Self::Geo => "S",
            Self::QZSS => "J",
            Self::IRNSS => "I",
            Self::Mixed => "M",
        }
    }
    /* Converts self to 3 letter code (RINEX standard code) */
    pub(crate) fn to_3_letter_code(&self) -> &str {
        match self {
            Self::GPS => "GPS",
            Self::Glonass => "GLO",
            Self::Galileo => "GAL",
            Self::BeiDou => "BDS",
            Self::SBAS(_) | Self::Geo => "GEO",
            Self::QZSS => "QZS",
            Self::IRNSS => "IRN",
            Self::Mixed => "MIX",
        }
    }

    /// Returns associated time scale. Returns None
    /// if related time scale is not supported.
    pub fn timescale(&self) -> Option<TimeScale> {
        match self {
            Self::GPS | Self::QZSS => Some(TimeScale::GPST),
            Self::Galileo => Some(TimeScale::GST),
            Self::BeiDou => Some(TimeScale::BDT),
            Self::Geo | Self::SBAS(_) => Some(TimeScale::GPST), // this is correct ?
            _ => None,
        }
    }
}

impl std::str::FromStr for Constellation {
    type Err = ParsingError;
    /// Identifies `gnss` constellation from given code.   
    /// Code should be standard constellation name,
    /// or official 1/3 letter RINEX code.    
    /// This method is case insensitive
    fn from_str(code: &str) -> Result<Self, Self::Err> {
        if code.len() == 3 {
            Ok(Self::from_3_letter_code(code)?)
        } else if code.len() == 1 {
            Ok(Self::from_1_letter_code(code)?)
        } else if let Ok(s) = Self::from_plain_name(code) {
            Ok(s)
        } else if let Ok(sbas) = Augmentation::from_str(code) {
            Ok(Self::SBAS(sbas))
        } else {
            Err(ParsingError::Unknown(code.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hifitime::TimeScale;
    use std::str::FromStr;
    #[test]
    fn from_1_letter_code() {
        let c = Constellation::from_1_letter_code("G");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::GPS);

        let c = Constellation::from_1_letter_code("R");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Glonass);

        let c = Constellation::from_1_letter_code("M");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Mixed);

        let c = Constellation::from_1_letter_code("J");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::QZSS);

        let c = Constellation::from_1_letter_code("X");
        assert_eq!(c.is_err(), true);
    }
    #[test]
    fn from_3_letter_code() {
        let c = Constellation::from_3_letter_code("GPS");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::GPS);
        let c = Constellation::from_3_letter_code("GLO");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Constellation::Glonass);
        let c = Constellation::from_3_letter_code("GPX");
        assert_eq!(c.is_err(), true);
        let c = Constellation::from_3_letter_code("X");
        assert_eq!(c.is_err(), true);
    }
    #[test]
    fn augmentation() {
        let c = Augmentation::from_str("WAAS");
        assert_eq!(c.is_ok(), true);
        assert_eq!(c.unwrap(), Augmentation::WAAS);
        let c = Augmentation::from_str("WASS");
        assert_eq!(c.is_err(), true);
    }
    #[test]
    fn timescale() {
        for (gnss, expected) in vec![
            (Constellation::GPS, TimeScale::GPST),
            (Constellation::Galileo, TimeScale::GST),
            (Constellation::BeiDou, TimeScale::BDT),
        ] {
            assert_eq!(gnss.timescale(), Some(expected));
        }
    }
}
