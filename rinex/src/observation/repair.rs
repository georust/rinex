//! Observation RINEX repair implementation
use crate::observation::Record;

#[cfg(feature = "processing")]
pub(crate) fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, (_, svnn)| {
        svnn.retain(|_, obs| {
            obs.retain(|ob, value| {
                if ob.is_pseudorange_observable() || ob.is_phase_observable() {
                    value.obs > 0.0
                } else {
                    true
                }
            });
            !obs.is_empty()
        });
        !svnn.is_empty()
    });
}

#[cfg(feature = "processing")]
pub(crate) fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}