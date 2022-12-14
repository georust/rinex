use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct QcOpts {
    /// Moving average window duration
    /// dedicated to RX clock drift estimate
    pub clk_drift_window: Duration,
    /// Moving average window duration
    /// dedicated to plain observation studies,
    /// like SSI average
    pub obs_avg_window: Duration,
    /// gap_considered:
    ///   + when set to None, we compare the epoch intervals
    ///   to the expected sample rate
    ///
    ///   + when set to a specific duration, we disregard
    ///   the expected sample rate and mark a gap anytime
    ///   instantaneous epoch interval exceeds this duration
    pub gap_considered: Option<Duration>,
    /// elevation mask 
    pub elev_mask: f64,
    /// elevation angle increment, in the augmented study
    pub elev_increment: f64,
    /// ionospheric induced variations tolerance
    pub max_iono_rate_cm_min: f64,
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            clk_drift_window: Duration::from_hours(1.0),
            obs_avg_window: Duration::from_hours(4.0),
            gap_considered: None,
            elev_mask: 10.0_f64,
            elev_increment: 10.0_f64,
            max_iono_rate_cm_min: 400.0,
        }
    }
}
