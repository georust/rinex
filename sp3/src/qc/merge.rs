use crate::prelude::SP3;

#[cfg(feature = "qc")]
use qc_traits::{Merge, MergeError};

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.agency != rhs.agency {
            return Err(MergeError::DataProviderAgencyMismatch);
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

#[cfg(feature = "processing")]
impl Preprocessing for SP3 {}

#[cfg(feature = "processing")]
impl Masking for SP3 {
    fn mask(&self, f: &MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(&f);
        s
    }
    fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch == *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| svs.contains(&k.sv));
                },
                FilterItem::ConstellationItem(constells) => {
                    let mut broad_sbas_filter = false;
                    for c in constells {
                        broad_sbas_filter |= *c == Constellation::SBAS;
                    }
                    self.data.retain(|k, _| {
                        if broad_sbas_filter {
                            k.sv.constellation.is_sbas() || constells.contains(&k.sv.constellation)
                        } else {
                            constells.contains(&k.sv.constellation)
                        }
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch != *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| !svs.contains(&k.sv));
                },
                FilterItem::ConstellationItem(constells) => {
                    self.data
                        .retain(|k, _| !constells.contains(&k.sv.constellation));
                },
                _ => {}, // does not apply
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch > *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn > sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch >= *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn >= sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch < *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn < sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    self.data.retain(|k, _| k.epoch <= *epoch);
                },
                FilterItem::SvItem(svs) => {
                    self.data.retain(|k, _| {
                        let mut retain = false;
                        for sv in svs {
                            if k.sv.constellation == sv.constellation {
                                retain = k.sv.prn <= sv.prn
                            } else {
                                retain = false
                            }
                        }
                        retain
                    });
                },
                _ => {}, // does not apply
            },
        }
    }
}

#[cfg(feature = "processing")]
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
