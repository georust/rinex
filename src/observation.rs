//! `RinexType::ObservationData` specific module
use thiserror::Error;
use std::str::FromStr;
use strum_macros::EnumString;
use physical_constants::SPEED_OF_LIGHT_IN_VACUUM;

use crate::version::RinexVersion;
use crate::constellation::Constellation;
use crate::record::{Epoch, Sv, RecordItemError};

#[macro_export]
/// Returns True if 3 letter code 
/// matches a pseudo range (OBS) code
macro_rules! is_pseudo_range_obs_code {
    ($code: expr) => { 
        $code.starts_with("C") || $code.starts_with("P") // non gps old fashion
    };
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

/// Calculates distance from given Pseudo Range value,
/// by compensating clock offsets    
/// pr: raw pseudo range measurements   
/// rcvr_clock_offset: receiver clock offset (s)    
/// sat_clock_offset: Sv clock offset (s)    
/// biases: other additive biases
pub fn distance_from_pseudo_range (pr: f64, 
    rcvr_clock_offset: f64, sat_clock_offset: f64, biases: Vec<f64>)
        -> f64
{
    pr - SPEED_OF_LIGHT_IN_VACUUM * (rcvr_clock_offset - sat_clock_offset)
    // modulo leap second?
    // p17 table 4
}

pub enum Error {
    UnknownRinexCode(String),
}

#[derive(EnumString)]
pub enum CarrierFrequency {
    /// L1 is a GPS/QZSS/Sbas carrier
    L1, 
    /// L2 is a GPS/QZSS carrier
    L2,
    /// L5 is a GPS/QZSS/Sbas/IRNSS carrier
    L5,
    /// L6 is a QZSS carrier
    L6,
    /// S is a IRNSS carrier
    S,
    /// E1 is a Galileo carrier
    E1,
    /// E5a is a Galileo carrier
    E5a,
    /// E5b is a Galileo carrier
    E5b,
    /// E5(E5a+E5b) is a Galileo carrier
    E5,
    /// E6 is a Galileo carrier
    E6,
    /// B1 is a Beidou carrier
    B1,
    /// B1c is a Beidou carrier
    B1c,
    /// B1a is a Beidou carrier
    B1a,
    /// B2a is a Beidou carrier
    B2a,
    /// B2b is a Beidou carrier
    B2b,
    /// B2(B2a+B2b) is a Beidou carrier
    B2,
    /// B3 is a Beidou carrier
    B3,
    /// B3a is a Beidou carrier
    B3a,
    /// G1 is a Glonass channel,
    G1(f64),
    /// G1a is a Glonass channel,
    G1a,
    /// G2 is a Glonass channel,
    G2(f64),
    /// G2a is a Glonass channel,
    G2a,
    /// G3 is a Glonass channel,
    G3,
}

impl CarrierFrequency {
    /// Returns carrier frequency [MHz]
    pub fn frequency (&self) -> f64 {
        match self {
            CarrierFrequency::L1 => 1575.42_f64,
            CarrierFrequency::L2 => 1227.60_f64,
            CarrierFrequency::L5 => 1176.45_f64,
            CarrierFrequency::L6 => 1278.75_f64,
            CarrierFrequency::S => 2492.028_f64,
            CarrierFrequency::E1 => 1575.42_f64,
            CarrierFrequency::E5a => 1176.45_f64,
            CarrierFrequency::E5b => 1207.140_f64,
            CarrierFrequency::E5 => 1191.795_f64,
            CarrierFrequency::E6 => 1278.75_f64,
            CarrierFrequency::B1 => 1561.098_f64,
            CarrierFrequency::B1c => 1575.42_f64,  
            CarrierFrequency::B1a => 1575.42_f64, 
            CarrierFrequency::B2a => 1176.45_f64, 
            CarrierFrequency::B2b => 1207.140_f64, 
            CarrierFrequency::B2 => 1191.795_f64, 
            CarrierFrequency::B3 => 1268.52_f64,
            CarrierFrequency::B3a => 1268.52_f64,
            CarrierFrequency::G1(c) => 1602.0_f64 + c *9.0/16.0, 
            CarrierFrequency::G1a => 1600.995_f64,
            CarrierFrequency::G2(c) => 1246.06_f64 + c* 7.0/16.0,
            CarrierFrequency::G2a => 1248.06_f64, 
            CarrierFrequency::G3 => 1202.025_f64,
        }
    }
}

pub enum SignalStrength {
    DbHz12, // < 12 dBc/Hz
    DbHz12_17, // 12 <= x < 17 dBc/Hz
    DbHz18_23, // 18 <= x < 23 dBc/Hz
    DbHz21_29, // 24 <= x < 29 dBc/Hz
    DbHz30_35, // 30 <= x < 35 dBc/Hz
    DbHz36_41, // 36 <= x < 41 dBc/Hz
    DbHz42_47, // 42 <= x < 47 dBc/Hz
    DbHz48_53, // 48 <= x < 53 dBc/Hz
    DbHz54, // >= 54 dBc/Hz 
}

/*impl SignalStrength {
    from f64::
}*/

/// `ObservationType` related errors
#[derive(Error, Debug)]
pub enum ObservationTypeError {
    #[error("obs code not recognized \"{0}\"")]
    UnknownObsCode(String),
}

/// Describes different kind of `Observations`
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ObservationType {
    /// Carrier phase range from antenna
    /// to Sv measured in whole cycles.    
    // 5.2.13
    /// Phase observations between epochs
    /// must be connected by including # integer cycles.   
    /// Phase obs. must be corrected for phase shifts
    ObservationPhase,
    /// Positive doppler means Sv is approaching
    ObservationDoppler,
    /// Pseudo Range is distance (m) from the
    /// receiver antenna to the Sv antenna,
    /// including clock offsets and other biases
    /// such as delays induced by atmosphere
    ObservationPseudoRange,
    /// Carrier signal strength observation
    ObservationSigStrength,
}

impl Default for ObservationType {
    fn default() -> ObservationType { ObservationType::ObservationPseudoRange }
}

impl std::str::FromStr for ObservationType {
    type Err = ObservationTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if is_pseudo_range_obs_code!(s) { 
            Ok(ObservationType::ObservationPseudoRange)
        } else if is_phase_carrier_obs_code!(s) {
            Ok(ObservationType::ObservationPhase)
        } else if is_doppler_obs_code!(s) {
            Ok(ObservationType::ObservationDoppler)
        } else if is_sig_strength_obs_code!(s) {
            Ok(ObservationType::ObservationSigStrength)
        } else {
            Err(ObservationTypeError::UnknownObsCode(s.to_string()))
        }
    }
}

/// Returns Observation record entries from given string content
pub fn build_obs_entry (version: RinexVersion,
    constellation: Constellation, content: &str)
        -> Result<(Sv,Epoch,Vec<f64>), RecordItemError> 
{
    Err(RecordItemError::UnknownTypeDescriptor(String::from("null")))
}

mod test {
    use super::*;
    #[test]
    /// Tests `CarrierFrequency` constructor
    fn test_carrier_frequency() {
        assert_eq!(CarrierFrequency::from_str("L1").is_err(),  false);
        assert_eq!(CarrierFrequency::from_str("E5a").is_err(), false);
        assert_eq!(CarrierFrequency::from_str("E7").is_err(),  true);
        assert_eq!(CarrierFrequency::from_str("L1").unwrap().frequency(), 1575.42_f64);
        assert_eq!(CarrierFrequency::from_str("G1a").unwrap().frequency(), 1600.995_f64);
    }
}
