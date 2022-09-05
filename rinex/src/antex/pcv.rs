//! Antex - special RINEX type specific structures
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown pcv code \"{0}\"")]
    UnknownPcv(String),
}

/// Antenna Phase Center Variation types
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Pcv {
    /// Given data is aboslute
    Absolute,
    /// Given data is relative
    Relative,
}

impl Default for Pcv {
    fn default() -> Self {
        Self::Absolute
    }
}

impl std::str::FromStr for Pcv {
    type Err = Error;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("A") {
            Ok(Self::Absolute)
        } else if content.eq("R") {
            Ok(Self::Relative)
        } else {
            Err(Error::UnknownPcv(content.to_string()))
        }
    }
}
