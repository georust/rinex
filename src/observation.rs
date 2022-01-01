//! `RinexType::ObservationData` specific module
use thiserror::Error;
use std::str::FromStr;

use crate::version::RinexVersion;
use crate::constellation::{Constellation, ConstellationError};

#[macro_export]
/// Returns True if 3 letter code 
/// matches a pseudo range (OBS) code
macro_rules! is_pseudo_range_obs_code {
    ($code: expr) => { $code.starts_with("C") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a phase (OBS) code
macro_rules! is_phase_carrier_obs_code {
    ($code: expr) => { $code.starts_with("L") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a doppler (OBS) code
macro_rules! is_doppler_obs_code {
    ($code: expr) => { $code.starts_with("D") };
}

#[macro_export]
/// Returns True if 3 letter code 
/// matches a signal strength (OBS) code
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
