//! Meteo specific Header fields
use std::io::{BufWriter, Write};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

use crate::{meteo::Sensor, prelude::Observable};

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
    /// Formats [HeaderFields] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        write!(w, "{:6}", self.codes.len())?;

        for (nth, observable) in self.codes.iter().enumerate() {
            if (nth % 9) == 8 {
                write!(w, "# / TYPES OF OBSERV\n      ")?;
            }
            write!(w, "    {}", observable)?;
        }

        for sensor in self.sensors.iter() {
            writeln!(w, "{}", sensor)?;
        }

        Ok(())
    }
}

#[cfg(feature = "processing")]
impl HeaderFields {
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
