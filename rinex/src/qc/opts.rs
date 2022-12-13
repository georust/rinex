use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct QcOpts {
    pub clk_drift_window: Duration,
    pub obs_avg_window: Duration,
    pub gap_considered: Option<Duration>,
    pub elev_mask: f64,
    pub elev_increment: f64,
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            clk_drift_window: Duration::from_hours(1.0),
            obs_avg_window: Duration::from_hours(1.0),
            gap_considered: None,
            elev_mask: 10.0_f64,
            elev_increment: 10.0_f64,
        }
    }
}
