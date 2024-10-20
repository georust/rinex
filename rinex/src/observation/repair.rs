//! Observation RINEX repair implementation
use crate::observation::Record;

use qc_traits::processing::Repair;

/// Repairs all Zero (=null) values in [Record]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, v| {
        if let Some(sig) = v.as_signal_mut() {
            if sig.observable.is_pseudo_range_observable()
                || sig.observable.is_phase_range_observable()
            {
                sig.value > 0.0
            } else {
                true
            }
        } else {
            true
        }
    });
}

/// Applies [Repair] to [Record]
pub fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}
