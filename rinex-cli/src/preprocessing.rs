use itertools::Itertools;
use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex::preprocessing::*;
use rinex::prelude::{RnxContext, Epoch};

use sp3::prelude::{DataType as SP3DataType, SP3};

/*
 * SP3 toolkit does not implement the Processing Traits
 * since they're currently defined in RINEX..
 * Work around this by implementing the ""typical"" preprocessing ops
 * manually here. This allows to shrink the SP3 context, which
 * is quite heavy, and make future Epoch iterations much quicker
 */
fn sp3_filter_mut(filter: Filter, sp3: &mut SP3) {
    match filter {
        Filter::Mask(mask) => sp3_mask_mut(mask, sp3),
        Filter::Decimation(decim) => sp3_decimate_mut(decim, sp3),
        _ => {}, // does not apply
    }
}

fn sp3_mask_mut(mask: MaskFilter, sp3: &mut SP3) {
    match mask.operand {
        MaskOperand::Equals => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t == epoch);
                sp3.clock_rate.retain(|t, _| *t == epoch);
                sp3.position.retain(|t, _| *t == epoch);
                sp3.velocities.retain(|t, _| *t == epoch);
            },
            TargetItem::ConstellationItem(constells) => {
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| constells.contains(&sv.constellation));
                    !data.is_empty()
                });
            },
            TargetItem::SvItem(svs) => {
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| svs.contains(sv));
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| svs.contains(sv));
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| svs.contains(sv));
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| svs.contains(sv));
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
        MaskOperand::NotEquals => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t != epoch);
                sp3.clock_rate.retain(|t, _| *t != epoch);
                sp3.position.retain(|t, _| *t != epoch);
                sp3.velocities.retain(|t, _| *t != epoch);
            },
            TargetItem::ConstellationItem(constells) => {
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| !constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| !constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| !constells.contains(&sv.constellation));
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| !constells.contains(&sv.constellation));
                    !data.is_empty()
                });
            },
            TargetItem::SvItem(svs) => {
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| !svs.contains(sv));
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| !svs.contains(sv));
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| !svs.contains(sv));
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| !svs.contains(sv));
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
        MaskOperand::GreaterEquals => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t >= epoch);
                sp3.clock_rate.retain(|t, _| *t >= epoch);
                sp3.position.retain(|t, _| *t >= epoch);
                sp3.velocities.retain(|t, _| *t >= epoch);
            },
            TargetItem::SvItem(svs) => {
                let constells = svs
                    .iter()
                    .map(|sv| sv.constellation)
                    .unique()
                    .collect::<Vec<_>>();
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                >= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                >= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                >= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                >= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
        MaskOperand::GreaterThan => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t > epoch);
                sp3.clock_rate.retain(|t, _| *t > epoch);
                sp3.position.retain(|t, _| *t > epoch);
                sp3.position.retain(|t, _| *t > epoch);
            },
            TargetItem::SvItem(svs) => {
                let constells = svs
                    .iter()
                    .map(|sv| sv.constellation)
                    .unique()
                    .collect::<Vec<_>>();
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                > svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                > svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                > svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                > svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
        MaskOperand::LowerThan => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t < epoch);
                sp3.clock_rate.retain(|t, _| *t < epoch);
                sp3.position.retain(|t, _| *t < epoch);
                sp3.velocities.retain(|t, _| *t < epoch);
            },
            TargetItem::SvItem(svs) => {
                let constells = svs
                    .iter()
                    .map(|sv| sv.constellation)
                    .unique()
                    .collect::<Vec<_>>();
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                < svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                < svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.position.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                < svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.velocities.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                < svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
        MaskOperand::LowerEquals => match mask.item {
            TargetItem::EpochItem(epoch) => {
                sp3.clock.retain(|t, _| *t <= epoch);
                sp3.clock_rate.retain(|t, _| *t <= epoch);
                sp3.position.retain(|t, _| *t <= epoch);
                sp3.velocities.retain(|t, _| *t <= epoch);
            },
            TargetItem::SvItem(svs) => {
                let constells = svs
                    .iter()
                    .map(|sv| sv.constellation)
                    .unique()
                    .collect::<Vec<_>>();
                sp3.clock.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                <= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.clock_rate.retain(|_t, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                <= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.position.retain(|_, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                <= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
                sp3.velocities.retain(|_, data| {
                    data.retain(|sv, _| {
                        constells.contains(&sv.constellation)
                            && sv.prn
                                <= svs
                                    .iter()
                                    .filter(|svs| svs.constellation == sv.constellation)
                                    .reduce(|k, _| k)
                                    .unwrap()
                                    .prn
                    });
                    !data.is_empty()
                });
            },
            _ => {}, // does not apply
        },
    }
}

fn sp3_decimate_mut(decim: DecimationFilter, sp3: &mut SP3) {
    match decim.dtype {
        DecimationType::DecimByRatio(r) => {
            let mut i = 0;
            sp3.clock.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
            let mut i = 0;
            sp3.clock_rate.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
            let mut i = 0;
            sp3.position.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
            let mut i = 0;
            sp3.velocities.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationType::DecimByInterval(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            sp3.clock.retain(|t, _| {
                if let Some(last) = last_retained {
                    let dt = *t - last;
                    if dt >= interval {
                        last_retained = Some(*t);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*t);
                    true // always retain 1st Epoch
                }
            });
            let mut last_retained = Option::<Epoch>::None;
            sp3.clock_rate.retain(|t, _| {
                if let Some(last) = last_retained {
                    let dt = *t - last;
                    if dt >= interval {
                        last_retained = Some(*t);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*t);
                    true // always retain 1st Epoch
                }
            });
            let mut last_retained = Option::<Epoch>::None;
            sp3.position.retain(|t, _| {
                if let Some(last) = last_retained {
                    let dt = *t - last;
                    if dt >= interval {
                        last_retained = Some(*t);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*t);
                    true // always retain 1st Epoch
                }
            });
            let mut last_retained = Option::<Epoch>::None;
            sp3.velocities.retain(|t, _| {
                if let Some(last) = last_retained {
                    let dt = *t - last;
                    if dt >= interval {
                        last_retained = Some(*t);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*t);
                    true // always retain 1st Epoch
                }
            });
        },
    }
}

/*
 * Once SP3 payload has been reworked,
 * we rework its header fields to the remaining payload.
 * This keeps file header consistent and allows for example
 * to generate a new SP3 that is consistent and correct.
 */
pub fn sp3_rework_mut(sp3: &mut SP3) {
    let svs = sp3
        .sv_position()
        .map(|(_, sv, _)| sv)
        .unique()
        .collect::<Vec<_>>();
    sp3.sv.retain(|sv| svs.contains(sv));

    let epochs = sp3
        .sv_position()
        .map(|(t, _, _)| t)
        .unique()
        .collect::<Vec<_>>();
    sp3.epoch.retain(|t| epochs.contains(t));

    if sp3.data_type == SP3DataType::Velocity && sp3.sv_velocities().count() == 0 {
        // dropped all Velocity information
        sp3.data_type = SP3DataType::Position;
    }
}

pub fn preprocess(ctx: &mut RnxContext, cli: &Cli) {
    // GNSS filters
    let mut gnss_filters = Vec::<&str>::new();

    if cli.gps_filter() {
        gnss_filters.push("!=gps");
        trace!("applying -G filter..");
    }
    if cli.glo_filter() {
        gnss_filters.push("!=glo");
        trace!("applying -R filter..");
    }
    if cli.gal_filter() {
        gnss_filters.push("!=gal");
        trace!("applying -E filter..");
    }
    if cli.bds_filter() {
        gnss_filters.push("!=bds");
        trace!("applying -C filter..");
    }
    if cli.sbas_filter() {
        gnss_filters.push("!=geo");
        trace!("applying -S filter..");
    }
    if cli.qzss_filter() {
        gnss_filters.push("!=qzss");
        trace!("applying -J filter..");
    }
    if cli.irnss_filter() {
        gnss_filters.push("!=irnss");
        trace!("applying -I filter..");
    }

    for filter in gnss_filters {
        let filter = Filter::from_str(filter).unwrap(); // cannot fail
        if let Some(inner) = ctx.observation_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(inner) = ctx.brdc_navigation_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(inner) = ctx.clock_mut() {
            inner.filter_mut(filter.clone());
        }
        if let Some(inner) = ctx.sp3_mut() {
            sp3_filter_mut(filter, inner);
        }
    }

    for filt_str in cli.preprocessing() {
        /*
         * Apply all preprocessing filters
         */
        if let Ok(filter) = Filter::from_str(filt_str) {
            if let Some(ref mut inner) = ctx.observation_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.brdc_navigation_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.meteo_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.clock_mut() {
                inner.filter_mut(filter.clone());
            }
            if let Some(ref mut inner) = ctx.sp3_mut() {
                sp3_filter_mut(filter.clone(), inner);
            }
            if let Some(ref mut _inner) = ctx.ionex_mut() {
                // FIXME: conclude IONEX preprocessing.
                // Time framing most importantly, will be useful
            }
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }

    /*
     * see [sp3_rework_mut]
     */
    if let Some(ref mut inner) = ctx.sp3_mut() {
        if !cli.preprocessing().is_empty() {
            sp3_rework_mut(inner);
        }
    }
}
