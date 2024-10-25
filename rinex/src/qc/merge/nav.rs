use crate::{navigation::Record, prelude::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (rhs_epoch, rhs_frames) in rhs {
        if let Some(frames) = rec.get_mut(rhs_epoch) {
            // this epoch already exists
            for fr in rhs_frames {
                if !frames.contains(fr) {
                    frames.push(fr.clone()); // insert new NavFrame
                }
            }
        } else {
            // insert new epoch
            rec.insert(*rhs_epoch, rhs_frames.clone());
        }
    }
    Ok(())
}
