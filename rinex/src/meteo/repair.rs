//! Meteo RINEX repair implementation
use crate::meteo::Record;
use qc_traits::Repair;

fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, value| *value != 0.0);
}

pub fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}
