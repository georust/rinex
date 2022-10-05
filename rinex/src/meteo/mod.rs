//! `Meteo` RINEX related structures & methods
pub mod sensor;
pub mod record;
pub mod observable;

pub use record::{
    Record,
    is_new_epoch,
    parse_epoch,
    write_epoch,
};

/// Meteo specific header fields
#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<observable::Observable>, 
    /// Sensors that produced the following observables
    pub sensors: Vec<sensor::Sensor>,
}

