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
