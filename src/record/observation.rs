//! Observation.rs
//! to describe `Rinex` file body content
//! for ObservationData files
use thiserror::Error;
use std::str::FromStr;

use crate::record::*;
use crate::version::RinexVersion;
use crate::constellation::{Constellation, ConstellationError};

#[macro_export]
/// Returns True if 3 letter code 
/// matches a pseudo range obs. code
macro_rules! is_pseudo_range_obs_code {
    ($code: expr) => { $code.starts_with("C") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a phase obs. code
macro_rules! is_phase_carrier_obs_code {
    ($code: expr) => { $code.starts_with("L") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a doppler obs. code
macro_rules! is_doppler_obs_code {
    ($code: expr) => { $code.starts_with("D") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a signal strenght obs. code
macro_rules! is_sig_strength_obs_code {
    ($code: expr) => { $code.starts_with("S") };
}

/// Describes different kind of `Observations`
#[derive(Debug)]
pub enum ObservationType {
    ObservationPhase,
    ObservationDoppler,
    ObservationPseudoRange,
    ObservationSigStrength,
}

/// Describes OBS records specific errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify constellation")]
    ParseConstellationError(#[from] ConstellationError),
    #[error("failed to build record item")]
    RecordItemError(#[from] RecordItemError),
}

/// `ObservationRecord` describes an OBS message frame.   
#[derive(Debug)]
pub struct ObservationRecord {
    items: std::collections::HashMap<String, RecordItem>,
}

impl Default for ObservationRecord {
    fn default() -> ObservationRecord {
        ObservationRecord {
            items: std::collections::HashMap::with_capacity(RECORD_MAX_SIZE),
        }
    }
}

impl ObservationRecord {
    /// Builds an `ObservationRecord` from raw record content 
    pub fn from_string (version: RinexVersion, 
            constellation: Constellation, s: &str) 
                -> Result<ObservationRecord, Error> 
    {
        let mut lines = s.lines();
        let mut line = lines.next()
            .unwrap();
        Ok(ObservationRecord::default())
    }
}
