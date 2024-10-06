//! [Decimate] implementation

use crate::prelude::{
    RINEX,
    Decimate, DecimationFilter,
};

impl Decimate for RINEX {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(f);
        s
    }
    fn decimate_mut(&mut self, f: &DecimationFilter) {
        if let Some(rec) = self.record.as_mut_obs() {
            observation_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_nav() {
            navigation_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_clock() {
            clock_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_meteo() {
            meteo_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_doris() {
            doris_decim_mut(rec, f)
        } else if let Some(rec) = self.record.as_mut_ionex() {
            ionex_decim_mut(rec, f)
        }
    }
}