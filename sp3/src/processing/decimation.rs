use qc_traits::{Decimate, DecimationFilter, DecimationFilterType};

use crate::prelude::{Epoch, Header, SP3};

impl Decimate for Header {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(f);
        s
    }
    fn decimate_mut(&mut self, f: &DecimationFilter) {
        match f.filter {
            DecimationFilterType::Duration(interval) => {
                self.epoch_interval = std::cmp::max(self.epoch_interval, interval);
            },
            DecimationFilterType::Modulo(modulo) => {
                self.epoch_interval = self.epoch_interval * modulo as f64;
            },
        }
    }
}

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
        self.header.decimate_mut(f);

        match f.filter {
            DecimationFilterType::Modulo(modulo) => {
                let mut i = 0;
                self.data.retain(|_, _| {
                    let retained = (i % modulo) == 0;
                    i += 1;
                    retained
                });
            },
            DecimationFilterType::Duration(interval) => {
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
