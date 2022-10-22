//! `ObservationData` parser and related methods
use super::{
    Constellation,
    version::Version,
};

pub mod record;
pub use record::{
    Record, LliFlags, Ssi,
    is_new_epoch,
    fmt_epoch,
    parse_epoch,
};

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::Serialize;

#[cfg(feature = "serde")]
use crate::formatter::datetime;

/// Describes `Compact RINEX` specific information
#[derive(Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Crinex {
    /// Compression program version
    pub version: Version,
    /// Compression program name
    pub prog: String,
    /// Date of compression
    #[cfg_attr(feature = "serde", serde(with = "datetime"))]
    pub date: chrono::NaiveDateTime,
}

impl Default for Crinex {
    fn default() -> Self {
        Self {
            version: Version {
                major: 3,
                minor: 0,
            },
            prog: "rust-crinex".to_string(),
            date: chrono::Utc::now().naive_utc(),
        }
    }
}

impl std::fmt::Display for Crinex {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let version = self.version.to_string();
        write!(f, "{:<width$}", version, width=20)?;
        write!(f, "{:<width$}", "COMPACT RINEX FORMAT", width=20)?;
        write!(f, "{value:<width$} CRINEX VERS   / TYPE\n", value="", width=19)?;
        write!(f, "{:<width$}", self.prog, width=20)?;
        write!(f, "{:20}", "")?;
        let date = self.date.format("%d-%b-%y %H:%M");
        write!(f, "{:<width$}", date, width=20)?;
        f.write_str("CRINEX PROG / DATE\n")
    }
}

/// Describes known marker types
/// Observation Record specific header fields
#[derive(Debug, Clone)]
#[derive(PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information,
    /// only present on compressed OBS
    pub crinex: Option<Crinex>, 
    /// Observation codes present in this file, by Constellation
    pub codes: HashMap<Constellation, Vec<String>>,
    /// True if epochs & data compensate for local clock drift
    pub clock_offset_applied: bool,
}
