use crate::prelude::{Constellation, Header, SP3};

use qc_traits::{Merge, MergeError};

impl Merge for Header {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError>
    where
        Self: Sized,
    {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        // Verifications
        if self.agency != rhs.agency {
            return Err(MergeError::DataProviderMismatch);
        }
        if self.time_scale != rhs.time_scale {
            return Err(MergeError::TimescaleMismatch);
        }
        if self.coord_system != rhs.coord_system {
            return Err(MergeError::ReferenceFrameMismatch);
        }

        // "upgrade" constellation
        if self.constellation != rhs.constellation {
            self.constellation = Constellation::Mixed;
        }

        // update revision
        self.version = std::cmp::min(self.version, rhs.version);

        // update time reference

        if rhs.mjd < self.mjd {
            self.mjd = rhs.mjd;
        }

        if rhs.week_counter < self.week_counter {
            self.week_counter = rhs.week_counter;
            self.week_sow = rhs.week_sow;
        }

        // update SV table
        for satellite in rhs.satellites.iter() {
            if !self.satellites.contains(&satellite) {
                self.satellites.push(*satellite);
            }
        }

        // update sampling
        self.epoch_interval = std::cmp::max(self.epoch_interval, rhs.epoch_interval);

        Ok(())
    }
}

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        self.header.merge_mut(&rhs.header)?;

        for (key, entry) in &rhs.data {
            if let Some(lhs_entry) = self.data.get_mut(key) {
                if let Some(clock_us) = entry.clock_us {
                    lhs_entry.clock_us = Some(clock_us);
                }

                if let Some(drift_ns) = entry.clock_drift_ns {
                    lhs_entry.clock_drift_ns = Some(drift_ns);
                }

                if let Some((vel_x_km_s, vel_y_km_s, vel_z_km_s)) = entry.velocity_km_s {
                    lhs_entry.velocity_km_s = Some((vel_x_km_s, vel_y_km_s, vel_z_km_s));
                }
            } else {
                self.data.insert(key.clone(), entry.clone()); // new entry
            }
        }
        Ok(())
    }
}
