//! `GNSS` constellations & associated methods
use thiserror::Error;
use serde_derive::{Deserialize, Serialize};

/// Number of known ̀`GNSS` constellations
pub const CONSTELLATION_LENGTH: usize = 6;

#[derive(Error, Debug)]
/// Constellation parsing & identification related errors
pub enum Error {
    #[error("code length mismatch, expecting {0} got {1}")]
    CodeLengthMismatch(usize,usize),
    #[error("unknown constellation code \"{0}\"")]
    UnknownCode(String),
}

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
    /// `Geo` constellation
    Geo,
    /// `IRNSS` constellation
    Irnss,
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

impl Constellation {
    /// Identifies `gnss` constellation from given 1 letter code.    
    /// Given code should match official RINEX codes.    
    /// This method is case insensitive though
    pub fn from_1_letter_code (code: &str) -> Result<Constellation, Error> {
        if code.len() != 1 {
            return Err(Error::CodeLengthMismatch(1, code.len()))
        }
        if code.to_lowercase().eq("g") {
            Ok(Constellation::GPS)
        } else if code.to_lowercase().eq("r") {
            Ok(Constellation::Glonass)
        } else if code.to_lowercase().eq("c") {
            Ok(Constellation::Beidou)
        } else if code.to_lowercase().eq("e") {
            Ok(Constellation::Galileo)
        } else if code.to_lowercase().eq("j") {
            Ok(Constellation::QZSS)
        } else if code.to_lowercase().eq("h") {
            Ok(Constellation::Sbas)
        } else if code.to_lowercase().eq("s") {
            Ok(Constellation::Geo)
        } else if code.to_lowercase().eq("i") {
            Ok(Constellation::Irnss)
        } else if code.to_lowercase().eq("m") {
            Ok(Constellation::Mixed)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
    /// Converts self to 1 letter code (RINEX standard code)
    pub fn to_1_letter_code (&self) -> &str {
        match self {
            Constellation::GPS => "G",
            Constellation::Glonass => "R",
            Constellation::Galileo => "E",
            Constellation::Beidou => "C",
            Constellation::Sbas => "H",
            Constellation::Geo => "S",
            Constellation::QZSS => "J",
            Constellation::Irnss => "I",
            Constellation::Mixed => "M",
        } 
    }
    /// Identifies `gnss` constellation from given 3 letter code.    
    /// Given code should match official RINEX codes.    
    /// This method is case insensitive though
    pub fn from_3_letter_code (code: &str) -> Result<Constellation, Error> {
        if code.len() != 3 {
            return Err(Error::CodeLengthMismatch(3, code.len()))
        }
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
        } else if code.to_lowercase().eq("geo") {
            Ok(Constellation::Geo)
        } else if code.to_lowercase().eq("irn") {
            Ok(Constellation::Irnss)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
    /// Converts self to 3 letter code (RINEX standard code)
    pub fn to_3_letter_code (&self) -> &str {
        match self {
            Constellation::GPS => "GPS",
            Constellation::Glonass => "GLO",
            Constellation::Galileo => "GAL",
            Constellation::Beidou => "BDS",
            Constellation::Sbas => "SBS",
            Constellation::Geo => "GEO",
            Constellation::QZSS => "QZS",
            Constellation::Irnss => "IRN",
            Constellation::Mixed => "MIX",
        } 
    }
    /// Identifies `gnss` constellation from given standard plain name,
    /// like "GPS", or "Galileo". This method is not case sensitive.
    pub fn from_plain_name (code: &str) -> Result<Constellation, Error> {
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
        } else if code.to_lowercase().contains("geo") {
            Ok(Constellation::Geo)
        } else if code.to_lowercase().contains("irnss") {
            Ok(Constellation::Irnss)
        } else if code.to_lowercase().contains("mixed") {
            Ok(Constellation::Mixed)
        } else {
            Err(Error::UnknownCode(code.to_string()))
        }
    }
}

impl std::str::FromStr for Constellation {
    type Err = Error;
    /// Identifies `gnss` constellation from given code.   
    /// Code should be standard constellation name,
    /// or official 1/3 letter RINEX code.    
    /// This method is case insensitive
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        if code.len() == 3 {
            Ok(Constellation::from_3_letter_code(code)?)
        } else if code.len() == 1 {
            Ok(Constellation::from_1_letter_code(code)?)
        } else {
            Ok(Constellation::from_plain_name(code)?)
        }
    }
}

#[derive(EnumString)]
/// Carrier frequency representation
pub enum CarrierCode {
    /// L1 is a GPS/QZSS/Sbas carrier
    L1, 
    /// L2 is a GPS/QZSS carrier
    L2,
    /// L5 is a GPS/QZSS/Sbas/IRNSS carrier
    L5,
    /// L6 is a QZSS carrier
    L6,
    /// S is a IRNSS carrier
    S,
    /// E1 is a Galileo carrier
    E1,
    /// E5a is a Galileo carrier
    E5a,
    /// E5b is a Galileo carrier
    E5b,
    /// E5(E5a+E5b) is a Galileo carrier
    E5,
    /// E6 is a Galileo carrier
    E6,
    /// B1 is a Beidou carrier
    B1,
    /// B1c is a Beidou carrier
    B1c,
    /// B1a is a Beidou carrier
    B1a,
    /// B2a is a Beidou carrier
    B2a,
    /// B2b is a Beidou carrier
    B2b,
    /// B2(B2a+B2b) is a Beidou carrier
    B2,
    /// B3 is a Beidou carrier
    B3,
    /// B3a is a Beidou carrier
    B3a,
    /// Glonass C1 carrier for given channel 
    /// use 0 for unknown ?
    C1(u8),
    /// C2 Glonass carrier for given channel
    C2(u8),
}

impl CarrierCode {
    /// Returns frequency of given CarrierCode
    /// in Hz
    pub fn to_frequency (&self) -> f64 {
        self.to_frequency_mhz() * 1E6
    }

    /// Returns frequency of given Carrier
    /// in MHz
    pub fn to_frequency_mhz (&self) -> f64 {
        match self {
            CarrierCode::L1 | CarrierCode::E1 | CarrierCode::B1c | CarrierCode::B1a => 1575.42_f64,
            CarrierCode::L5 | CarrierCode::E5a | CarrierCode::B2a => 1176.45_f64,
            
            CarrierCode::B2 | CarrierCode::E5 => 1191.795_f64, 
            CarrierCode::E5b | CarrierCode::B2b => 1207.140_f64,
            CarrierCode::L2  => 1227.60_f64,
            CarrierCode::C2(c) => 1246.06_f64 + (*c as f64 * 7.0/16.0),
            CarrierCode::B3 | CarrierCode::B3a => 1268.52_f64,
            CarrierCode::L6 | CarrierCode::E6 => 1278.75_f64,
            CarrierCode::B1 => 1561.098_f64,
            CarrierCode::C1(c) => 1602.0_f64 + (*c as f64 *9.0/16.0), 
            CarrierCode::S   => 2492.028_f64,
        }
    }
    
    /// Returns frequency of given CarrierCode
    /// in Hz
    pub fn to_frequency_ghz (&self) -> f64 {
        self.to_frequency_mhz() * 1E-3
    }
}

mod test {
    use super::*;
    #[test]
    /// Tests `CarrierCode` constructor
    fn test_carrier_code() {
        assert_eq!(super::CarrierCode::from_str("L1").is_err(),  false);
        assert_eq!(super::CarrierCode::from_str("E5a").is_err(), false);
        assert_eq!(super::CarrierCode::from_str("E7").is_err(),  true);
        assert_eq!(super::CarrierCode::from_str("L1").unwrap().frequency(), 1575.42_f64);
    }
}
