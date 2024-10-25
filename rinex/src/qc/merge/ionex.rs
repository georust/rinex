use crate::{ionex::Record, prelude::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (eh, plane) in rhs {
        if let Some(lhs_plane) = rec.get_mut(eh) {
            for (latlon, plane) in plane {
                if let Some(tec) = lhs_plane.get_mut(latlon) {
                    if let Some(rms) = plane.rms {
                        if tec.rms.is_none() {
                            tec.rms = Some(rms);
                        }
                    }
                } else {
                    lhs_plane.insert(*latlon, plane.clone());
                }
            }
        } else {
            rec.insert(*eh, plane.clone());
        }
    }
    Ok(())
}
