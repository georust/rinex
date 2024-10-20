#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [ClockObservation] is the GNSS receiver clock observation.
/// Might be present in OBS RINEX records.
#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockObservation {
    /// Clock offset to GNSS constellation, which is defined in
    /// Header section.
    pub offset_s: f64,
}
