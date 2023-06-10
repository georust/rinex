use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub trait IonoDelayDetector {
    /// Evaluates Ionospheric delay detector for all signals and vehicles
    fn iono_delay_detector(
        &self,
        dt: Duration,
    ) -> HashMap<Observable, HashMap<Sv, BTreeMap<Epoch, f64>>>;
}
