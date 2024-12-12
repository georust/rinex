//! IONEX module
use std::collections::BTreeMap;

mod formatting;
mod grid;
mod header;
mod ipp;
mod parsing;
mod quantized;
mod rinex;
mod system;

#[cfg(feature = "processing")]
mod decim;

#[cfg(feature = "processing")]
mod mask;

#[cfg(feature = "processing")]
mod repair;

#[cfg(feature = "processing")]
pub(crate) use decim::decim_mut;

#[cfg(feature = "processing")]
pub(crate) use mask::mask_mut;

#[cfg(feature = "processing")]
pub(crate) use repair::repair_mut;

pub use grid::Grid;
pub use header::HeaderFields;
pub use ipp::IPPCoordinates;
pub use system::RefSystem;

pub(crate) use parsing::{
    is_new_height_map, is_new_rms_map, is_new_tec_map, parse_height_map, parse_rms_map,
    parse_tec_map,
};

pub(crate) use quantized::Quantized;

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

/// Modeled Ionosphere characteristics
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct IonosphereParameters {
    /// Amplitude of the ionospheric delay (s)
    pub amplitude_s: f64,
    /// Period of the ionospheric delay (s)
    pub period_s: f64,
    /// Phase of the ionospheric delay (rad)
    pub phase_rad: f64,
    /// Slant factor is the projection factor
    /// from a vertical signal propagation,
    /// to actual angled shifted propagation (no unit)
    pub slant_factor: f64,
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
    /// TEC quantized in TEcu
    tecu: Quantized,
    /// RMS (TEC)
    rms: Option<Quantized>,
}

impl TEC {
    /// Builds new [TEC] from TEC estimate expressed in TECu (=10^16 m-2)
    pub fn from_tecu(tecu: f64) -> Self {
        let exponent = Quantized::find_exponent(tecu);
        Self {
            tecu: Quantized::new(tecu, exponent),
            rms: None,
        }
    }

    /// Builds new [TEC] from raw TEC estimate in m^-2
    pub fn from_tec_m2(tec: f64) -> Self {
        let tecu = tec / 10.0E16;
        let exponent = Quantized::find_exponent(tecu);
        Self {
            tecu: Quantized::new(tecu, exponent),
            rms: None,
        }
    }

    /// Builds new [TEC] estimate with associated RMS value
    pub fn with_rms(&self, rms: f64) -> Self {
        let mut s = self.clone();
        let exponent = Quantized::find_exponent(rms);
        s.rms = Some(Quantized::new(rms, exponent));
        s
    }

    /// Builds new [TEC] from TEC quantization in TECu
    pub(crate) fn from_quantized(tecu: i32, exponent: i8) -> Self {
        // IONEX stores quantized TEC as i=10*-k TECu
        Self {
            tecu: Quantized {
                exponent: -exponent,
                quantized: tecu,
            },
            rms: None,
        }
    }

    /// Updates RMS [TEC]
    pub(crate) fn set_quantized_rms(&mut self, rms: i32, exponent: i8) {
        self.rms = Some(Quantized {
            exponent: -exponent,
            quantized: rms,
        });
    }

    /// Returns Total Electron Content estimate, in TECu (=10^-16 m-2)
    pub fn tecu(&self) -> f64 {
        self.tecu.real_value()
    }

    /// Returns Total Electron Content estimate, in m-2
    pub fn tec(&self) -> f64 {
        self.tecu() * 10.0E16
    }

    /// Returns Root Mean Square (TEC) if it was provided.
    pub fn rms_tec(&self) -> Option<f64> {
        let rms = self.rms?;
        Some(rms.real_value())
    }
}

#[cfg(feature = "qc")]
use qc_traits::{Merge, MergeError};

#[cfg(feature = "qc")]
impl Merge for TEC {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(&rhs)?;
        Ok(s)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.rms.is_none() {
            self.rms = rhs.rms.clone();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IonexMapCoordinates {
    /// Quantized latitude
    latitude: Quantized,
    /// Quantized longitude
    longitude: Quantized,
    /// Quantized altitude
    altitude: Quantized,
}

impl IonexMapCoordinates {
    /// Builds new [IonexMapCoordinates] from coordinates expressed in ddeg
    #[cfg(test)]
    pub fn new(
        lat_ddeg: f64,
        lat_exponent: i8,
        long_ddeg: f64,
        long_exponent: i8,
        alt_km: f64,
        alt_exponent: i8,
    ) -> Self {
        Self {
            latitude: Quantized::new(lat_ddeg, lat_exponent),
            longitude: Quantized::new(long_ddeg, long_exponent),
            altitude: Quantized::new(alt_km, alt_exponent),
        }
    }

    /// Builds new [IonexMapCoordinates] from [Quantized] coordinates
    pub(crate) fn from_quantized(
        latitude: Quantized,
        longitude: Quantized,
        altitude: Quantized,
    ) -> Self {
        Self {
            latitude,
            longitude,
            altitude,
        }
    }

    /// Returns latitude in degrees
    pub fn latitude_ddeg(&self) -> f64 {
        self.latitude.real_value()
    }

    /// Returns longitude in degrees
    pub fn longitude_ddeg(&self) -> f64 {
        self.longitude.real_value()
    }
    /// Returns longitude in kilometers
    pub fn altitude_km(&self) -> f64 {
        self.altitude.real_value()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

    #[test]
    fn quantized_ionex_map_coords() {
        let coords = IonexMapCoordinates::new(1.0, 1, 2.0, 1, 3.0, 1);
        assert_eq!(coords.latitude_ddeg(), 1.0);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.0);

        let coords = IonexMapCoordinates::new(1.5, 1, 2.0, 1, 3.12, 2);
        assert_eq!(coords.latitude_ddeg(), 1.5);
        assert_eq!(coords.longitude_ddeg(), 2.0);
        assert_eq!(coords.altitude_km(), 3.12);
    }

    #[test]
    fn quantized_tec() {
        let tec = TEC::from_quantized(30, -1);
        assert_eq!(tec.tecu(), 3.0);
        assert_eq!(tec.tec(), 3.0 * 10E16);

        let tec = TEC::from_quantized(30, -2);
        assert_eq!(tec.tecu(), 0.3);
        assert_eq!(tec.tec(), 0.3 * 10E16);

        let tec = TEC::from_tec_m2(1.0 * 10E16);
        assert_eq!(tec.tecu(), 1.0);
        assert_eq!(tec.tec(), 1.0 * 10E16);
    }
}
