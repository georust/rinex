#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Clock Profile is the actual measurement or estimate
/// at a specified Epoch.
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockProfile {
    /// Clock bias [s]
    pub bias: f64,
    /// Clock bias deviation
    pub bias_dev: Option<f64>,
    /// Clock drift [s/s]
    pub drift: Option<f64>,
    /// Clock drift deviation
    pub drift_dev: Option<f64>,
    /// Clock drift change [s/s^2]
    pub drift_change: Option<f64>,
    /// Clock drift change deviation
    pub drift_change_dev: Option<f64>,
}

/// Type of Observation/Measurement
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ClockProfileType {
    /// Data analysis results for receiver clocks
    /// derived from a set of network receivers and satellites
    AR,
    /// Data analysis results for satellites clocks
    /// derived from a set of network receivers and satellites
    AS,
    /// Single GNSS Receiver Calibration
    CR,
    /// Discontinuuous Single GNSS Receiver Calibration
    DR,
    /// Broadcast SV clocks monitoring
    MS,
}

impl std::fmt::Display for ClockProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AR => f.write_str("AR"),
            Self::AS => f.write_str("AS"),
            Self::CR => f.write_str("CR"),
            Self::DR => f.write_str("DR"),
            Self::MS => f.write_str("MS"),
        }
    }
}
