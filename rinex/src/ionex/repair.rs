use crate::ionex::Record;

use qc_traits::Repair;

/// Repairs all Zero (=null) values in [Record]
fn repair_zero_mut(rec: &mut Record) {
    rec.retain(|_, tec| tec.tec() > 0.0);
}

/// Applies [Repair] to [Record]
pub fn repair_mut(rec: &mut Record, repair: Repair) {
    match repair {
        Repair::Zero => repair_zero_mut(rec),
    }
}
