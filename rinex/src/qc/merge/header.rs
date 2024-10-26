use crate::prelude::{Constellation, Epoch, Header, Merge, MergeError};

use super::{
    merge_mut_option, merge_mut_unique_map2d, merge_mut_unique_vec, merge_mut_vec,
    merge_time_of_first_obs, merge_time_of_last_obs,
};

impl Merge for Header {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        if self.rinex_type != rhs.rinex_type {
            return Err(MergeError::FileTypeMismatch);
        }
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.rinex_type != rhs.rinex_type {
            return Err(MergeError::FileTypeMismatch);
        }

        let (a_cst, b_cst) = (self.constellation, rhs.constellation);
        if a_cst != b_cst {
            // <=> Constellation "upgrade"
            self.constellation = Some(Constellation::Mixed)
        }

        // retain oldest revision
        let (a_rev, b_rev) = (self.version, rhs.version);
        self.version = std::cmp::min(a_rev, b_rev);

        // sampling interval special case
        match self.sampling_interval {
            None => {
                if rhs.sampling_interval.is_some() {
                    self.sampling_interval = rhs.sampling_interval;
                }
            },
            Some(lhs) => {
                if let Some(rhs) = rhs.sampling_interval {
                    self.sampling_interval = Some(std::cmp::min(lhs, rhs));
                }
            },
        }

        merge_mut_vec(&mut self.comments, &rhs.comments);
        merge_mut_option(&mut self.geodetic_marker, &rhs.geodetic_marker);
        merge_mut_option(&mut self.license, &rhs.license);
        merge_mut_option(&mut self.doi, &rhs.doi);
        merge_mut_option(&mut self.leap, &rhs.leap);
        merge_mut_option(&mut self.gps_utc_delta, &rhs.gps_utc_delta);
        merge_mut_option(&mut self.rcvr, &rhs.rcvr);
        merge_mut_option(&mut self.cospar, &rhs.cospar);
        merge_mut_option(&mut self.rcvr_antenna, &rhs.rcvr_antenna);
        merge_mut_option(&mut self.sv_antenna, &rhs.sv_antenna);
        merge_mut_option(&mut self.ground_position, &rhs.ground_position);
        merge_mut_option(&mut self.wavelengths, &rhs.wavelengths);
        merge_mut_option(&mut self.gps_utc_delta, &rhs.gps_utc_delta);

        // DCBS compensation is preserved, only if both A&B both have it
        if self.dcb_compensations.is_empty() || rhs.dcb_compensations.is_empty() {
            self.dcb_compensations.clear(); // drop everything
        } else {
            let rhs_constellations: Vec<_> = rhs
                .dcb_compensations
                .iter()
                .map(|dcb| dcb.constellation)
                .collect();
            self.dcb_compensations
                .iter_mut()
                .filter(|dcb| rhs_constellations.contains(&dcb.constellation))
                .count();
        }

        // PCV compensation : same logic
        // only preserve compensations present in both A & B
        if self.pcv_compensations.is_empty() || rhs.pcv_compensations.is_empty() {
            self.pcv_compensations.clear(); // drop everything
        } else {
            let rhs_constellations: Vec<_> = rhs
                .pcv_compensations
                .iter()
                .map(|pcv| pcv.constellation)
                .collect();
            self.dcb_compensations
                .iter_mut()
                .filter(|pcv| rhs_constellations.contains(&pcv.constellation))
                .count();
        }

        // TODO: merge::merge_mut(&mut self.glo_channels, &rhs.glo_channels);

        // RINEX specific operation
        if let Some(lhs) = &mut self.antex {
            if let Some(rhs) = &rhs.antex {
                // ANTEX records can only be merged together
                // if they have the same type of inner phase data
                let mut mixed_antex = lhs.pcv_type.is_relative() && !rhs.pcv_type.is_relative();
                mixed_antex |= !lhs.pcv_type.is_relative() && rhs.pcv_type.is_relative();
                if mixed_antex {
                    return Err(MergeError::FileTypeMismatch);
                }
                //TODO: merge_mut_option(&mut lhs.reference_sn, &rhs.reference_sn);
            }
        }
        if let Some(lhs) = &mut self.clock {
            if let Some(rhs) = &rhs.clock {
                merge_mut_unique_vec(&mut lhs.codes, &rhs.codes);
                merge_mut_option(&mut lhs.igs, &rhs.igs);
                merge_mut_option(&mut lhs.site, &rhs.site);
                merge_mut_option(&mut lhs.domes, &rhs.domes);
                merge_mut_option(&mut lhs.full_name, &rhs.full_name);
                merge_mut_option(&mut lhs.ref_clock, &rhs.ref_clock);
                merge_mut_option(&mut lhs.timescale, &rhs.timescale);
            }
        }
        if let Some(lhs) = &mut self.obs {
            if let Some(rhs) = &rhs.obs {
                merge_mut_option(&mut lhs.crinex, &rhs.crinex);
                merge_mut_unique_map2d(&mut lhs.codes, &rhs.codes);
                merge_time_of_first_obs(&mut lhs.timeof_first_obs, &rhs.timeof_first_obs);
                merge_time_of_last_obs(&mut lhs.timeof_last_obs, &rhs.timeof_last_obs);
                // TODO: lhs.clock_offset_applied |= rhs.clock_offset_applied;
            }
        }
        if let Some(lhs) = &mut self.meteo {
            if let Some(rhs) = &rhs.meteo {
                merge_mut_unique_vec(&mut lhs.codes, &rhs.codes);
                merge_mut_unique_vec(&mut lhs.sensors, &rhs.sensors);
            }
        }
        if let Some(lhs) = &mut self.doris {
            if let Some(rhs) = &rhs.doris {
                merge_time_of_first_obs(&mut lhs.time_of_first_obs, &rhs.time_of_first_obs);
                merge_time_of_last_obs(&mut lhs.time_of_last_obs, &rhs.time_of_last_obs);
                merge_mut_unique_vec(&mut lhs.stations, &rhs.stations);
                merge_mut_unique_vec(&mut lhs.observables, &rhs.observables);
                //TODO: merge_scaling();
                //merge_mut_unique_map2d(&mut lhs.scaling, &rhs.scaling);
                lhs.l2_l1_date_offset = std::cmp::max(lhs.l2_l1_date_offset, rhs.l2_l1_date_offset);
            }
        }
        if let Some(lhs) = &mut self.ionex {
            if let Some(rhs) = &rhs.ionex {
                if lhs.reference != rhs.reference {
                    return Err(MergeError::ReferenceFrameMismatch);
                }
                if lhs.grid != rhs.grid {
                    return Err(MergeError::DimensionMismatch);
                }
                if lhs.map_dimension != rhs.map_dimension {
                    return Err(MergeError::DimensionMismatch);
                }
                if lhs.base_radius != rhs.base_radius {
                    return Err(MergeError::DimensionMismatch);
                }

                //TODO: this is not enough, need to take into account and rescale..
                lhs.exponent = std::cmp::min(lhs.exponent, rhs.exponent);

                merge_mut_option(&mut lhs.description, &rhs.description);
                merge_mut_option(&mut lhs.mapping, &rhs.mapping);
                if lhs.elevation_cutoff == 0.0 {
                    // means "unknown"
                    lhs.elevation_cutoff = rhs.elevation_cutoff; // => overwrite in this case
                }
                merge_mut_option(&mut lhs.observables, &rhs.observables);
                lhs.nb_stations = std::cmp::max(lhs.nb_stations, rhs.nb_stations);
                lhs.nb_satellites = std::cmp::max(lhs.nb_satellites, rhs.nb_satellites);
                for (b, dcb) in &rhs.dcbs {
                    lhs.dcbs.insert(b.clone(), *dcb);
                }
            }
        }

        // add special comment
        let now = Epoch::now().map_err(|_| MergeError::Other)?;

        let merge_comment = Self::merge_comment(now);
        self.comments.push(merge_comment);
        Ok(())
    }
}
