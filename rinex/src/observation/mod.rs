//! `ObservationData` parser and related methods
use std::collections::HashMap;
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

use crate::version;
use crate::constellation::Constellation;

pub mod record;

#[cfg(feature = "with-serde")]
use serde::Serialize;

#[cfg(feature = "with-serde")]
use crate::formatter::datetime;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Crinex {
    /// Compression program version
    pub version: version::Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    #[cfg_attr(feature = "with-serde", serde(with = "datetime"))]
    pub date: chrono::NaiveDateTime,
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct HeaderFields {
    /// Optional CRINEX information,
    /// only present on compressed OBS
    pub crinex: Option<Crinex>, 
    /// Observation codes present in this file, by Constellation
    pub codes: HashMap<Constellation, Vec<String>>,
    /// True if epochs & data compensate for local clock drift
    pub clock_offset_applied: bool,
}

/// Calculates distance from given Pseudo Range value,
/// by compensating clock offsets    
/// pseudo_rg: raw pseudo range measurements   
/// rcvr_clock_offset: receiver clock offset (s)    
/// sv_clock_offset: Sv clock offset (s)    
/// biases: optionnal (additive) biases to compensate for and increase result accuracy 
pub fn pseudo_range_to_distance (pseudo_rg: f64, rcvr_clock_offset: f64, sv_clock_offset: f64, _biases: Vec<f64>) -> f64 {
    pseudo_rg - SPEED_OF_LIGHT_IN_VACUUM * (rcvr_clock_offset - sv_clock_offset)
    //TODO handle biases
    // p17 table 4
}
