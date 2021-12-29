use thiserror::Error;

pub const GPS_IDENTIFIER: char     = 'G'; 
pub const GLONASS_IDENTIFIER: char = 'R'; 
pub const GALILEO_IDENTIFIER: char = 'E'; 
pub const QZSS_IDENTIFIER: char    = 'J';
pub const BEIDOU_IDENTIFIER: char  = 'C';
pub const MIXED_IDENTIFIER: char   = 'M';
pub const SBAS_IDENTIFIER: char    = 'S';

/// Describes all known `GNSS` constellations
/// when manipulating `RINEX`
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Constellation {
    GPS,
    Glonass,
    Beidou,
    QZSS,
    Galileo,
    Mixed,
    Sbas,
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
        match s.chars().nth(0)
            .unwrap() {
            GPS_IDENTIFIER => Ok(Constellation::GPS),
            GLONASS_IDENTIFIER => Ok(Constellation::Glonass),
            GALILEO_IDENTIFIER => Ok(Constellation::Galileo),
            QZSS_IDENTIFIER => Ok(Constellation::QZSS),
            BEIDOU_IDENTIFIER => Ok(Constellation::Beidou),
            MIXED_IDENTIFIER => Ok(Constellation::Mixed),
            SBAS_IDENTIFIER => Ok(Constellation::Sbas),
            _ => Ok(Constellation::Glonass),
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
