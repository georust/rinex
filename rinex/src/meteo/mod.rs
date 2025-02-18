//! Meteo RINEX module
pub mod record;
pub use record::Record;

pub mod sensor;
use sensor::Sensor;

use crate::Observable;

#[cfg(feature = "processing")]
use std::str::FromStr;

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

/// Meteo specific header fields
#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Observation types contained in this file
    pub codes: Vec<Observable>,
    /// Sensors that produced the following observables
    pub sensors: Vec<Sensor>,
}

impl HeaderFields {
    #[cfg(feature = "processing")]
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    self.codes.retain(|c| observables.contains(&c));
                },
                _ => {},
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    self.codes.retain(|c| !observables.contains(&c));
                },
                _ => {},
            },
            _ => {},
        }
    }
}
