//! [Masking] implementation

/**
 * Data [Masking] is uterly important when it comes to processing
**/
use crate::{prelude::RINEX, MaskFilter, Masking};

impl Masking for RINEX {
    fn mask(&self, f: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(f);
        s
    }
    fn mask_mut(&mut self, f: &MaskFilter) {
        if let Some(rec) = self.record.as_mut_obs() {
            observation_mask_mut(rec, f);
        } else if let Some(rec) = self.record.as_mut_nav() {
            navigation_mask_mut(rec, f);
        } else if let Some(rec) = self.record.as_mut_clock() {
            clock_mask_mut(rec, f);
        } else if let Some(rec) = self.record.as_mut_meteo() {
            meteo_mask_mut(rec, f);
        } else if let Some(rec) = self.record.as_mut_doris() {
            doris_mask_mut(rec, f);
        } else if let Some(rec) = self.record.as_mut_ionex() {
            ionex_mask_mut(rec, f);
        }
        header_mask_mut(&mut self.header, f);
    }
}
