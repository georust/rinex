//! `Meteo` RINEX related structures & methods
pub mod sensor;
pub mod record;
pub mod observable;

/// Meteo specific header fields
#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<observable::Observable>, 
    /// Sensors that produced the following observables
    pub sensors: Vec<sensor::Sensor>,
}

