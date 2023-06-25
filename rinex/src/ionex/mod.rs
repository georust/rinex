use super::{Epoch, Sv};
use std::collections::HashMap;
use strum_macros::EnumString;

pub mod record;
pub use record::Record;

pub mod grid;
pub use grid::{Grid, GridLinspace};

pub mod system;
pub use system::RefSystem;

#[cfg(feature = "serde")]
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, PartialOrd, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Mapping function used in when determining this IONEX
pub enum MappingFunction {
    /// 1/cos(z)
    #[strum(serialize = "COSZ")]
    CosZ,
    /// Q-factor
    #[strum(serialize = "QFAC")]
    QFac,
}

/// Possible source of DCBs
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BiasSource {
    /// Referenced against a given vehicle
    SpaceVehicle(Sv),
    /// Referenced for an observation station on Earth
    Station(String),
}

/// `IONEX` specific header fields
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
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
    /// Map grid definition
    pub map_grid: grid::Grid,
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
            reference: RefSystem::default(),
            exponent: -1,     // very important: allows missing EXPONENT fields
            map_dimension: 2, // 2D map by default
            mapping: None,
            observables: None,
            description: None,
            elevation_cutoff: 0.0,
            base_radius: 0.0,
            map_grid: Grid::default(),
            nb_stations: 0,
            nb_satellites: 0,
            dcbs: HashMap::new(),
        }
    }
}

impl HeaderFields {
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
            d.push_str(" ");
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
        if o.len() > 0 {
            s.observables = Some(o.to_string())
        }
        s
    }
    /// Returns true if this Ionosphere Maps describes
    /// a theoretical model, not measured data
    pub fn is_theoretical_model(&self) -> bool {
        self.observables.is_some()
    }
    /// Copies self and set Nb of stations
    pub fn with_nb_stations(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_stations = n;
        s
    }
    /// Copies self and set Nb of satellites
    pub fn with_nb_satellites(&self, n: u32) -> Self {
        let mut s = self.clone();
        s.nb_satellites = n;
        s
    }
    /// Copies & set Base Radius [km]
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
    pub fn with_latitude_grid(&self, grid: GridLinspace) -> Self {
        let mut s = self.clone();
        s.map_grid.lat_grid = grid;
        s
    }
    /// Adds longitude grid definition
    pub fn with_longitude_grid(&self, grid: GridLinspace) -> Self {
        let mut s = self.clone();
        s.map_grid.lon_grid = grid;
        s
    }
    /// Adds altitude grid definition
    pub fn with_altitude_grid(&self, grid: GridLinspace) -> Self {
        let mut s = self.clone();
        s.map_grid.h_grid = grid;
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

pub trait Ionex {
    /// Returns all latitude coordinates in this dataset
    fn latitudes(&self) -> Vec<f64>;
    /// Returns all longitude coordinates in this dataset
    fn longitudes(&self) -> Vec<f64>;
    /// Returns TEC map absolute maximal value
    fn max(&self) -> (Epoch, f64, f64, f64, f64);
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_mapping_func() {
        let content = "COSZ";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_ok(), true);
        assert_eq!(func.unwrap(), MappingFunction::CosZ);
        let content = "QFAC";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_ok(), true);
        assert_eq!(func.unwrap(), MappingFunction::QFac);
        let content = "DONT";
        let func = MappingFunction::from_str(content);
        assert_eq!(func.is_err(), true);
    }
}
