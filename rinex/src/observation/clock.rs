use crate::prelude::Epoch;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [ClockObservation] represents the state of a clock with respect to a [TimeScale].
#[derive(Default, Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ClockObservation {
    /// Clock offset to GNSS constellation, which is defined in
    /// Header section.
    pub offset_s: f64,
    /// Clock drift in [s/s]
    pub drift_s_s: f64,
    /// Clock drift rate in [s/s^2]
    pub drift_rate_s_s2: f64,
    /// Previous [Epoch] of observation
    timeof_obs: Option<Epoch>,
}

impl ClockObservation {
    /// Update fields with new offset [s].
    pub fn set_offset_s(&mut self, timeof_obs: Epoch, offset_s: f64) {
        if let Some(past) = self.timeof_obs {
            let dt_s = (timeof_obs - past).to_seconds();
            let drift_s_s = self.drift_s_s;
            self.drift_s_s = (offset_s - self.offset_s) / dt_s;
            self.drift_rate_s_s2 = (self.drift_s_s - drift_s_s) / dt_s;
        } else {
            self.drift_s_s = 0.0;
            self.drift_rate_s_s2 = 0.0;
        }
        self.timeof_obs = Some(timeof_obs);
        self.offset_s = offset_s;
    }

    /// Copies and define a new [ClockObservation] with given offset [s]
    pub fn with_offset_s(&self, timeof_obs: Epoch, offset_s: f64) -> Self {
        let mut s = self.clone();
        s.set_offset_s(timeof_obs, offset_s);
        s
    }
}
