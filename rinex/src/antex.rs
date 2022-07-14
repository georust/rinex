//! Antex - special RINEX type specific structures
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
}

/// ANTEX special RINEX fields
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Antenna Phase Center Variations type 
    pub pcv: Pcv, 
    /// Types of relative values, default: "AOAD/M_T"
    pub relative_values: String,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_sn: Option<String>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            pcv: Pcv::default(),
            relative_values: String::new(),
            reference_sn: None,
        }
    }
}


/// Antenna Phase Center Variation types
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
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
