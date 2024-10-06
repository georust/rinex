//! Meteo RINEX module
pub mod record;
pub use record::Record;

pub mod sensor;
use sensor::Sensor;

use crate::Observable;

mod formater; // fmt_ helpers
mod parser; // parse_ helpers

/// Meteo [RINEX] entry
pub struct MeteoObservation {
    /// Observation [Epoch]
    pub epoch: Epoch,
    /// [Observable]
    pub observable: Observable,
    /// Measurement, unit is [Observable] dependent.
    pub value: f64,
}

#[cfg(feature = "processing")]
use std::str::FromStr;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

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

/**
 * macro used in parsing logic
 */
pub(crate) fn is_new_epoch(line: &str, v: version::Version) -> bool {
    if v.major < 3 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 4, minor: 0 }
        ));
    }
}
