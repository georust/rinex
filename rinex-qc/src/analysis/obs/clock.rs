//! RX clock page, during receiver analysis
use rinex::prelude::{obs::ClockObservation, Epoch};

#[derive(Default)]
pub struct ClockAnalysis {
    pub clock_offset_s: Vec<(Epoch, f64)>,
    pub clock_drift_s_s: Vec<(Epoch, f64)>,
}

impl ClockAnalysis {
    // False when this [ClockPage] should be rendered
    pub fn is_null(&self) -> bool {
        self.clock_offset_s.is_empty()
    }

    /// latch new measurement
    pub fn new_measurement(&mut self, t: Epoch, observation: &ClockObservation) {
        self.clock_drift_s_s.push((t, observation.offset_s));
        if let Some(drift_s_s) = observation.drift_s_s {
            self.clock_drift_s_s.push((t, drift_s_s));
        }
    }
}
