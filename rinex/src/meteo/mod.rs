//! `Meteo` RINEX
pub mod observable;
pub mod record;
pub mod sensor;
pub use observable::Observable;
pub use record::Record;

/// Meteo specific header fields
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<Observable>,
    /// Sensors that produced the following observables
    pub sensors: Vec<sensor::Sensor>,
}
