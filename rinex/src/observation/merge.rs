use crate::{
    merge::{Error, Merge},
    observation::Record,
};

/// [Record] Merge ops
impl Merge for Record {
    /// Merge `rhs` into `Self`
    fn merge(&self, rhs: &Self) -> Result<Self, Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merge `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), Error> {
        for (rhs_epoch, (rhs_clk, rhs_vehicles)) in rhs {
            if let Some((clk, vehicles)) = self.get_mut(rhs_epoch) {
                // exact epoch (both timestamp and flag) did exist
                //  --> overwrite clock field (as is)
                *clk = *rhs_clk;
                // other fields:
                // either insert (if did not exist), or overwrite
                for (rhs_vehicle, rhs_observations) in rhs_vehicles {
                    if let Some(observations) = vehicles.get_mut(rhs_vehicle) {
                        for (rhs_observable, rhs_data) in rhs_observations {
                            if let Some(data) = observations.get_mut(rhs_observable) {
                                *data = *rhs_data; // overwrite
                            } else {
                                // new observation: insert it
                                observations.insert(rhs_observable.clone(), *rhs_data);
                            }
                        }
                    } else {
                        // new SV: insert it
                        vehicles.insert(*rhs_vehicle, rhs_observations.clone());
                    }
                }
            } else {
                // this epoch did not exist previously: insert it
                self.insert(*rhs_epoch, (*rhs_clk, rhs_vehicles.clone()));
            }
        }
        Ok(())
    }
}
