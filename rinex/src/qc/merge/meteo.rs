use crate::{meteo::Record, prelude::qc::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (epoch, observations) in rhs.iter() {
        if let Some(oobservations) = rec.get_mut(epoch) {
            for (observation, data) in observations.iter() {
                if !oobservations.contains_key(observation) {
                    // new observation
                    oobservations.insert(observation.clone(), *data);
                }
            }
        } else {
            // new epoch
            rec.insert(*epoch, observations.clone());
        }
    }
    Ok(())
}
