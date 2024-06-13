//! Meteo RINEX module
pub mod record;
pub mod sensor;
pub use record::Record;

use crate::Observable;

#[cfg(feature = "processing")]
use itertools::Itertools;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

/// Meteo specific header fields
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<Observable>,
    /// Sensors that produced the following observables
    pub sensors: Vec<sensor::Sensor>,
}

impl HeaderFields {
    #[cfg(feature = "processing")]
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {},
            MaskOperand::NotEquals => match &f.item {},
            MaskOperand::GreaterThan => match &f.item {},
            MaskOperand::GreaterEquals => match &f.item {},
            MaskOperand::LowerThan => match &f.item {},
            MaskOperand::LowerEquals => match &f.item {},
        }
    }
}
