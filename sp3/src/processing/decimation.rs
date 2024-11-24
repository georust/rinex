use qc_traits::{
    Decimate,
    DecimationFilter,
    DecimationFilterType,
};

use crate::prelude::{SP3, Epoch};

impl Decimate for SP3 {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(&f);
        s
    }
    fn decimate_mut(&mut self, f: &DecimationFilter) {
        if f.item.is_some() {
            todo!("targetted decimation not supported yet");
        }
        match f.filter {
            DecimationFilterType::Modulo(r) => {
                self.epoch_interval = self.epoch_interval * r as f64;
                let mut i = 0;
                self.data.retain(|_, _| {
                    let retained = (i % r) == 0;
                    i += 1;
                    retained
                });
            },
            DecimationFilterType::Duration(interval) => {
                self.epoch_interval = interval;
                let mut last_retained = Option::<Epoch>::None;
                self.data.retain(|k, _| {
                    if let Some(last) = last_retained {
                        let dt = k.epoch - last;
                        if dt >= interval {
                            last_retained = Some(k.epoch);
                            true
                        } else {
                            false
                        }
                    } else {
                        last_retained = Some(k.epoch);
                        true
                    }
                });
            },
        }
    }
}
