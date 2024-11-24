use crate::prelude::{SP3, Constellation};

use qc_traits::{Merge, MergeError};

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.agency != rhs.agency {
            return Err(MergeError::DataProviderMismatch);
        }
        if self.time_scale != rhs.time_scale {
            return Err(MergeError::TimescaleMismatch);
        }
        if self.coord_system != rhs.coord_system {
            return Err(MergeError::ReferenceFrameMismatch);
        }
        if self.constellation != rhs.constellation {
            /*
             * Convert self to Mixed constellation
             */
            self.constellation = Constellation::Mixed;
        }
        // adjust revision
        if rhs.version > self.version {
            self.version = rhs.version;
        }
        // Adjust MJD start
        if rhs.mjd_start.0 < self.mjd_start.0 {
            self.mjd_start.0 = rhs.mjd_start.0;
        }
        if rhs.mjd_start.1 < self.mjd_start.1 {
            self.mjd_start.1 = rhs.mjd_start.1;
        }
        // Adjust week counter
        if rhs.week_counter.0 < self.week_counter.0 {
            self.week_counter.0 = rhs.week_counter.0;
        }
        if rhs.week_counter.1 < self.week_counter.1 {
            self.week_counter.1 = rhs.week_counter.1;
        }
        // update SV table
        for sv in &rhs.sv {
            if !self.sv.contains(sv) {
                self.sv.push(*sv);
            }
        }
        // update sampling interval (pessimistic)
        self.epoch_interval = std::cmp::max(self.epoch_interval, rhs.epoch_interval);
        // Merge new entries
        // and upgrade missing information (if possible)
        for (key, entry) in &rhs.data {
            if let Some(lhs_entry) = self.data.get_mut(key) {
                if let Some(clock) = entry.clock {
                    lhs_entry.clock = Some(clock);
                }
                if let Some(rate) = entry.clock_rate {
                    lhs_entry.clock_rate = Some(rate);
                }
                if let Some(velocity) = entry.velocity {
                    lhs_entry.velocity = Some(velocity);
                }
            } else {
                if !self.epoch.contains(&key.epoch) {
                    self.epoch.push(key.epoch); // new epoch
                }
                self.data.insert(key.clone(), entry.clone()); // new entry
            }
        }
        self.epoch.sort(); // preserve chronological order
        Ok(())
    }
}

