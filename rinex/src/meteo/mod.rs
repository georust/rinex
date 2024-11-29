//! Meteo RINEX module
mod formatting;
mod header;
mod parsing;
mod sensor;

pub use sensor::Sensor;

use crate::prelude::{Epoch, Observable};

#[cfg(feature = "processing")]
pub(crate) mod mask; // mask Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod decim; // decim Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod repair; // repair Trait implementation

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeteoKey {
    /// [Epoch] of observation
    pub epoch: Epoch,
    /// [Observable] determines the physics
    pub observable: Observable,
}

/// Meteo observation (unit depends on [Observable]), sorted by [MeteoKey]
pub type Record = BTreeMap<ObsKey, f64>;
