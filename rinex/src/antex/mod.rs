//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod antenna;
pub mod frequency;
pub mod pcv;

mod parser; // parse_* methods
mod formater; // fmt_* methods

pub use antenna::{
    Antenna, AntennaMatcher, AntennaSpecific, Calibration, CalibrationMethod, Cospar, RxAntenna,
    SvAntenna,
};

pub use entry::Entry;
pub use pcv::PCV;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Macro used when parsing ATX files
pub(crate) fn is_new_entry(content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

//! ANTEX record indexing
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// ANTEX [RINEX] payload
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Entry {
    /// [Antenna]
    pub antenna: Antenna,
    /// [Carrier] signal
    pub carrier: Carrier,
    /// Eccentricities of the mean APC as NEU coordinates in millimeters.
    /// The offset position is either relative to
    ///     * ARP: Antenna Reference Point in case of [RxAntenna],
    ///     * MC: Spacecraft Mass Center in case of [SvAntenna].
    pub apc_eccentricity: (f64, f64, f64),
    /// [AntennaPhasePattern].
    /// NB: we do not support Azimuth Dependent phase patterns at the moment.
    pub phase_pattern: AntennaPhasePattern,
}

/// Phase pattern description.
/// We currently do not support azimuth dependent phase patterns.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AntennaPhasePattern {
    /// Azimuth Independent Phase pattern
    AzimuthIndependentPattern(Vec<f64>),
}

impl Default for AntennaPhasePattern {
    fn default() -> Self {
        Self::AzimuthIndependentPattern(Vec::<f64>::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_entry() {
        let content = "                                                           START OF ANTENNA";
        assert!(is_new_entry(content));
        let content =
            "TROSAR25.R4      LEIT727259                                 TYPE / SERIAL NO";
        assert!(!is_new_epoch(content));
        let content =
            "    26                                                      # OF FREQUENCIES";
        assert!(!is_new_epoch(content));
        let content =
            "   G01                                                      START OF FREQUENCY";
        assert!(!is_new_epoch(content));
    }
}