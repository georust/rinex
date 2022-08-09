//! `ObservationData` parser and related methods
use std::collections::HashMap;
use crate::version;
use crate::constellation::Constellation;

pub mod record;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
use crate::formatter::datetime;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Crinex {
    /// Compression program version
    pub version: version::Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    #[cfg_attr(feature = "serde", serde(with = "datetime"))]
    pub date: chrono::NaiveDateTime,
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct HeaderFields {
    /// Optional CRINEX information,
    /// only present on compressed OBS
    pub crinex: Option<Crinex>, 
    /// Observation codes present in this file, by Constellation
    pub codes: HashMap<Constellation, Vec<String>>,
    /// True if epochs & data compensate for local clock drift
    pub clock_offset_applied: bool,
}
