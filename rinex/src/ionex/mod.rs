//! IONEX module
use crate::prelude::{Epoch, ParsingError, SV};
use std::collections::HashMap;

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

pub mod record;
pub use record::{Record, TECPlane, TEC};

pub mod grid;
use crate::linspace::Linspace;
pub use grid::Grid;

pub mod system;
pub use system::RefSystem;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

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

/// `IONEX` specific header fields
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Epoch of first map
    pub epoch_of_first_map: Epoch,
    /// Epoch of last map
    pub epoch_of_last_map: Epoch,
    /// Reference system used for following TEC maps,
    /// cf. [system::RefSystem].
    pub reference: RefSystem,
    /// It is highly recommended to give a brief description
    /// of the technique, model.. description is not a
    /// general purpose comment
    pub description: Option<String>,
    /// Mapping function adopted for TEC determination,
    /// if None: No mapping function, e.g altimetry
    pub mapping: Option<MappingFunction>,
    /// Maps dimension, can either be a 2D (= fixed altitude mode), or 3D
    pub map_dimension: u8,
    /// Mean earth radius or bottom of height grid, in km.
    pub base_radius: f32,
    /// Reference rid definition.
    pub grid: grid::Grid,
    /// Minimum elevation angle filter used. In degrees.
    pub elevation_cutoff: f32,
    /// Verbose description of observables used in determination.
    /// When no Observables were used, that means we're based off a theoretical model.
    pub observables: Option<String>,
    /// Number of stations that contributed to following data
    pub nb_stations: u32,
    /// Number of satellites that contributed to following data
    pub nb_satellites: u32,
    /// exponent: scaling to apply in current TEC blocs
    pub exponent: i8,
    /// Differential Code Biases (DBCs),
    /// per Vehicle #PRN, (Bias and RMS bias) values.
    pub dcbs: HashMap<BiasSource, (f64, f64)>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            epoch_of_first_map: Epoch::default(),
            epoch_of_last_map: Epoch::default(),
            reference: RefSystem::default(),
            exponent: -1,     // very important: allows missing EXPONENT fields
            map_dimension: 2, // 2D map by default
            mapping: None,
            observables: None,
            description: None,
            elevation_cutoff: 0.0,
            base_radius: 0.0,
            grid: grid::Grid::default(),
            nb_stations: 0,
            nb_satellites: 0,
            dcbs: HashMap::new(),
        }
    }
}

impl HeaderFields {
    /// Copies self with given time of first map
    pub fn with_epoch_of_first_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_first_map = t;
        s
    }
    /// Copies self with given time of last map
    pub fn with_epoch_of_last_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_last_map = t;
        s
    }
    /// Copies and builds Self with given Reference System
    pub fn with_reference_system(&self, reference: RefSystem) -> Self {
        let mut s = self.clone();
        s.reference = reference;
        s
    }
    /// Copies and sets exponent / scaling to currently use
    pub fn with_exponent(&self, e: i8) -> Self {
        let mut s = self.clone();
        s.exponent = e;
        s
    }
    /// Copies and sets model description
    pub fn with_description(&self, desc: &str) -> Self {
        let mut s = self.clone();
        if let Some(ref mut d) = s.description {
            d.push(' ');
            d.push_str(desc)
        } else {
            s.description = Some(desc.to_string())
        }
        s
    }
    pub fn with_mapping_function(&self, mf: MappingFunction) -> Self {
        let mut s = self.clone();
        s.mapping = Some(mf);
        s
    }
    /// Copies & sets minimum elevation angle used.
    pub fn with_elevation_cutoff(&self, e: f32) -> Self {
        let mut s = self.clone();
        s.elevation_cutoff = e;
        s
    }
    pub fn with_observables(&self, o: &str) -> Self {
        let mut s = self.clone();
        if !o.is_empty() {
            s.observables = Some(o.to_string())
        }
        s
    }
    /// Returns true if this Ionosphere Maps describes
    /// a theoretical model, not measured data
    pub fn is_theoretical_model(&self) -> bool {
        self.observables.is_some()
    }
    /// Copies self and set number of stations
    pub fn with_nb_stations(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_stations = n;
        s
    }
    /// Copies self and set number of satellites
    pub fn with_nb_satellites(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_satellites = n;
        s
    }
    /// Copies & set Base Radius in km
    pub fn with_base_radius(&self, b: f32) -> Self {
        let mut s = self.clone();
        s.base_radius = b;
        s
    }
    pub fn with_map_dimension(&self, d: u8) -> Self {
        let mut s = self.clone();
        s.map_dimension = d;
        s
    }
    /// Adds latitude grid definition
    pub fn with_latitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.latitude = grid;
        s
    }
    /// Adds longitude grid definition
    pub fn with_longitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.longitude = grid;
        s
    }
    /// Adds altitude grid definition
    pub fn with_altitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.height = grid;
        s
    }
    /// Copies & sets Diffenretial Code Bias estimates
    /// for given vehicle
    pub fn with_dcb(&self, src: BiasSource, value: (f64, f64)) -> Self {
        let mut s = self.clone();
        s.dcbs.insert(src, value);
        s
    }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    /// Timescale helper
    fn timescale(&self) -> TimeScale {
        self.epoch_of_first_map.time_scale
    }
    /// Modifies in place Self, when applying preprocessing filter ops
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::NotEquals => {},
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    self.epoch_of_first_map = epoch.to_time_scale(ts);
                    self.epoch_of_last_map = epoch.to_time_scale(ts);
                },
                FilterItem::SvItem(svs) => {
                    self.nb_satellites = svs.len() as u32;
                },
                _ => {},
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if self.epoch_of_first_map < *epoch {
                        self.epoch_of_first_map = epoch.to_time_scale(ts);
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if self.epoch_of_first_map < *epoch {
                        self.epoch_of_first_map = epoch.to_time_scale(ts);
                    }
                },
                _ => {},
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if self.epoch_of_last_map > *epoch {
                        self.epoch_of_last_map = epoch.to_time_scale(ts);
                    }
                },
                _ => {},
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if self.epoch_of_last_map > *epoch {
                        self.epoch_of_last_map = epoch.to_time_scale(ts);
                    }
                },
                _ => {},
            },
        }
    }
}

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
