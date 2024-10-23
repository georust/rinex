//! Observation RINEX masking ops

use crate::{
    observation::Record,
    prelude::{Constellation, Observable, SNR},
};

use rinex_qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

use std::str::FromStr;

/// Applies [MaskFilter] to [Record]
pub fn mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch == *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, obs| obs.clock.is_some());
            },
            FilterItem::ConstellationItem(constells) => {
                let mut broad_sbas_filter = false;
                for c in constells {
                    broad_sbas_filter |= *c == Constellation::SBAS;
                }
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if broad_sbas_filter {
                            sig.sv.constellation.is_sbas()
                                || constells.contains(&sig.sv.constellation)
                        } else {
                            constells.contains(&sig.sv.constellation)
                        }
                    } else {
                        true
                    }
                });
            },
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        items.contains(&sig.sv)
                    } else {
                        true
                    }
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if let Some(snr) = sig.snr {
                            snr == filter
                        } else {
                            false // no SNR: drop out
                        }
                    } else {
                        true // does not apply
                    }
                });
            },
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !observables.is_empty() {
                    rec.retain(|_, obs| {
                        if let Some(sig) = obs.as_signal() {
                            observables.contains(&sig.observable)
                        } else {
                            true
                        }
                    });
                }
            },
            _ => {},
        }, // MaskOperand::Equals

        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch != *epoch),
            FilterItem::ClockItem => {
                rec.retain(|_, obs| obs.as_clock().is_none());
            },
            FilterItem::ConstellationItem(constells) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        !constells.contains(&sig.sv.constellation)
                    } else {
                        true
                    }
                });
            },
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        !items.contains(&sig.sv)
                    } else {
                        true
                    }
                });
            },
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
                    .iter()
                    .filter_map(|f| {
                        if let Ok(ob) = Observable::from_str(f) {
                            Some(ob)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                if !observables.is_empty() {
                    rec.retain(|_, obs| {
                        if let Some(sig) = obs.as_signal() {
                            !observables.contains(&sig.observable)
                        } else {
                            true
                        }
                    });
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch >= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        let mut retained = true;
                        for item in items {
                            if item.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn >= item.prn;
                            }
                        }
                        retained
                    } else {
                        true
                    }
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if let Some(snr) = sig.snr {
                            snr >= filter
                        } else {
                            false // no SNR: drop out
                        }
                    } else {
                        true // does not apply
                    }
                });
            },
            _ => {},
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch > *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        let mut retained = true;
                        for item in items {
                            if item.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn > item.prn;
                            }
                        }
                        retained
                    } else {
                        true
                    }
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if let Some(snr) = sig.snr {
                            snr > filter
                        } else {
                            false // no SNR: drop out
                        }
                    } else {
                        true // does not apply
                    }
                });
            },
            _ => {},
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch <= *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        let mut retained = true;
                        for item in items {
                            if item.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn <= item.prn;
                            }
                        }
                        retained
                    } else {
                        true
                    }
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if let Some(snr) = sig.snr {
                            snr <= filter
                        } else {
                            false // no SNR: drop out
                        }
                    } else {
                        true // does not apply
                    }
                });
            },
            _ => {},
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|k, _| k.epoch < *epoch),
            FilterItem::SvItem(items) => {
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        let mut retained = true;
                        for item in items {
                            if item.constellation == sig.sv.constellation {
                                retained &= sig.sv.prn < item.prn;
                            }
                        }
                        retained
                    } else {
                        true
                    }
                });
            },
            FilterItem::SNRItem(filter) => {
                let filter = SNR::from(*filter);
                rec.retain(|_, obs| {
                    if let Some(sig) = obs.as_signal() {
                        if let Some(snr) = sig.snr {
                            snr < filter
                        } else {
                            false // no SNR: drop out
                        }
                    } else {
                        true // does not apply
                    }
                });
            },
            _ => {},
        },
    }
}
