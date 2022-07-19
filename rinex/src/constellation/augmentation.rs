//! `GNSS` constellations & associated methods
use thiserror::Error;
use strum_macros::EnumString;

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
/// GNSS Augmentation systems,
/// must be used based on current location
pub enum Augmentation {
    /// American augmentation system,
    WAAS,
    /// European augmentation system
    EGNOS,
    /// Japanese augmentation system
    MSAS,
    /// Indian augmentation system
    GAGAN,
    /// Chinese augmentation system
    BDSBAS,
    /// South Korean augmentation system
    KASS,
    /// Russian augmentation system
    SDCM,
    /// South African augmentation system
    ASBAS,
    /// Autralia / NZ augmentation system
    SPAN,
}

impl Default for Augmentation {
    fn default() -> Augmentation {
        Augmentation::WAAS
    }
}
