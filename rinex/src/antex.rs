//! Antex - special RINEX type specific structures
use thiserror::Error;
use crate::channel;
use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
}

/// Returns true if this line matches 
/// the beginning of a `epoch` for ATX file (special files),
/// this is not really an epoch but rather a group of dataset
/// for this given antenna, there is no sampling data attached to it.
pub fn is_new_epoch (content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

/// ANTEX Record content,
/// is a list of Antenna with Several `Frequency` items in it.
/// ATX record is not `epoch` iterable.
/// All `epochs_()` related methods would fail.
pub type Record = HashMap<Antenna, Vec<Frequency>>;

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

/// Describes an Antenna section inside the ATX record
pub struct Antenna {
    /// TODO
    pub ant_type: String,
    /// TODO
    pub sn: String,
    /// TODO
    pub method: Option<String>,
    /// TODO
    pub agency: Option<String>,
    /// TODO
    pub date: chrono::NaiveDate,
    /// TODO
    pub dazi: f64,
    /// TODO
    pub zen: (f64, f64),
    /// TODO
    pub dzen: f64,
    /// TODO
    pub valid_from: chrono::NaiveDateTime,
    /// TODO
    pub valid_until: chrono::NaiveDateTime,
}

pub enum Pattern {
    /// Non azimuth dependent pattern
    NonAzimuthDependent(Vec<f64>),
    /// Azimuth dependent pattern
    AzimuthDependent(Vec<f64>),
}

/// Describes a "frequency" section of the ATX record
pub struct Frequency {
    /// Channel, example: L1, L2 for GPS, E1, E5 for GAL...
    pub channel: channel::Channel,
    /// TODO
    pub north: f64,
    /// TODO
    pub east: f64,
    /// TODO
    pub up: f64,
    /// Possibly azimuth dependent pattern
    pub pattern: Pattern, 
}
