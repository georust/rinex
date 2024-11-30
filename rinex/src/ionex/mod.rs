//! IONEX module
use std::collections::BTreeMap;

mod formatting;
mod grid;
mod header;
mod parsing;
mod quantized;
mod system;
mod rinex;

#[cfg(feature = "processing")]
mod decim;

#[cfg(feature = "processing")]
mod mask;

#[cfg(feature = "processing")]
pub(crate) use decim::decim_mut;

#[cfg(feature = "processing")]
pub(crate) use mask::mask_mut;

pub use grid::Grid;
pub use header::HeaderFields;
pub use system::RefSystem;

use quantized::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Epoch, ParsingError, SV};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Mapping function used in when determining this IONEX
pub enum MappingFunction {
    /// 1/cos(z)
    CosZ,
    /// Q-factor
    QFac,
}

impl std::str::FromStr for MappingFunction {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "cosz" => Ok(Self::CosZ),
            "qfac" => Ok(Self::QFac),
            _ => Err(ParsingError::IonexMappingFunction),
        }
    }
}

impl std::fmt::Display for MappingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosZ => write!(f, "Cos(z)"),
            Self::QFac => write!(f, "Q-factor"),
        }
    }
}

/// Possible source of DCBs
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BiasSource {
    /// Referenced against a given vehicle
    SpaceVehicle(SV),
    /// Referenced for an observation station on Earth
    Station(String),
}

/// Total Electron Content (TEC) estimate
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TEC {
    /// TEC estimate
    pub tec: f64,
    /// RMS (TEC)
    pub rms: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct IonexMapCoordinates {
    /// Quantized latitude
    latitude: Quantized<i16>,
    /// Quantized longitude
    longitude: Quantized<i16>,
    /// Quantized altitude
    altitude: Quantized<i16>,
}

impl IonexMapCoordinates {
    /// Buildd new [IonexMapCoordinates]
    /// ## Input
    /// - lat_ddeg: latitude in decimal degrees
    /// - dlat: latitude grid spacing (degrees)
    /// - long_ddeg: longitude in decimal degrees
    /// - dlon: longitude grid spacing (degrees)
    /// - alt_km: altitude (km)
    /// - dalt: altitude grid spacing (km)
    pub fn new(
        lat_ddeg: f64,
        dlat: f64,
        long_ddeg: f64,
        dlon: f64,
        alt_km: f64,
        dalt: f64,
    ) -> Self {
        Self {
            latitude: Quantized::new(lat_ddeg, dlat),
            longitude: Quantized::new(long_ddeg, dlon),
            altitude: Quantized::new(alt_km, dalt),
        }
    }

    /// Returns latitude in degrees
    pub fn latitude_ddeg(&self) -> f64 {
        self.latitude.value()
    }

    /// Returns longitude in degrees
    pub fn longitude_ddeg(&self) -> f64 {
        self.longitude.value()
    }

    /// Returns longitude in kilometers
    pub fn altitude_km(&self) -> f64 {
        self.altitude.value()
    }

}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct IonexKey {
    /// [Epoch] of this [TEC] estimate.
    pub epoch: Epoch,
    /// [IonexMapCoordinates] to which the [TEC] estimate applies.
    pub coordinates: IonexMapCoordinates,
}

/// IONEX Record is [TEC] maps sorted by [IonexKey]
pub type Record = BTreeMap<IonexKey, TEC>;

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_mapping_func() {
        let content = "COSZ";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::CosZ);
        let content = "QFAC";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::QFac);
        let content = "DONT";
        let func = MappingFunction::from_str(content);
        assert!(func.is_err());
    }
}
