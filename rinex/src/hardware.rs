//! Hardware and receiver related structures
#[cfg(feature = "with-serde")]
use crate::formatter::point3d;

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

/// GNSS receiver description
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Receiver (hardware) model
    pub model: String, 
    /// Receiver (hardware) identification info
    pub sn: String, // serial #
    /// Receiver embedded software info
    pub firmware: String, // firmware #
}

impl Default for Rcvr {
    /// Builds a `default` Receiver
    fn default() -> Rcvr {
        Rcvr {
            model: String::new(),
            sn: String::new(),
            firmware: String::new(),
        }
    }
}

impl std::str::FromStr for Rcvr {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, rem) = line.split_at(20);
        let (make, rem) = rem.split_at(20);
        let (version, _) = rem.split_at(20);
        Ok(Rcvr{
            sn: id.trim().to_string(),
            model: make.trim().to_string(),
            firmware: version.trim().to_string(),
        })
    }
}

/// Antenna description 
#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Antenna {
    /// Hardware model / make descriptor
    pub model: String,
    /// Serial number / identification number
    pub sn: String,
    /// 3D coordinates of reference point
    #[cfg_attr(feature = "with-serde", serde(with = "point3d"))]
    pub coords: Option<rust_3d::Point3D>,
    /// height in comparison to ref. point
    pub height: Option<f32>,
    /// eastern eccentricity compared to ref. point
    pub eastern_ecc: Option<f32>,
    /// northern eccentricity compare to ref. point
    pub northern_ecc: Option<f32>,
}

impl Default for Antenna {
    /// Builds default `Antenna` structure
    fn default() -> Antenna {
        Antenna {
            model: String::new(),
            sn: String::new(),
            coords: None,
            height: None,
            eastern_ecc: None,
            northern_ecc: None,
        }
    }
}
