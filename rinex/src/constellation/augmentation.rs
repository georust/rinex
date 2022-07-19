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

#[cfg(feature = "with-geo")]
use geo::{polygon, Contains};

/// SBAS augmentation system selection helper,
/// returns most approriate Augmentation system
/// depending on given location (x,y) in ECEF [ddeg]
#[cfg(feature = "with-geo")]
pub fn sbas_selection_helper (pos: (f64,f64)) -> Option<Augmentation> {
    let point : geo::Point = pos.into();
    let boundaries :Vec<(Augmentation, geo::Polygon)> = vec![
        (Augmentation::WAAS, polygon![
            (x: 0.0, y: 0.0),
            (x: 10.0, y: 10.0),
            (x: -10.0, y: -10.0),
            (x: 0.0, y: 0.0),
        ]),
    ];
    for (sbas, area) in boundaries.iter() {
        if area.exterior().contains(&point) {
            return Some(sbas.clone())
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[cfg(feature = "with-geo")]
    fn test_sbas_selection() {
        // PARIS --> EGNOS
        let pos = (100.0, 200.0);
        assert_eq!(sbas_selection_helper(pos).is_some(), true);
    }
}
