use log::error;
use std::str::FromStr;

use crate::Cli;
use rinex_qc::prelude::{Filter, Preprocessing, QcContext};
use sp3::prelude::{DataType as SP3DataType, SP3};

//fn sp3_decimate_mut(decim: DecimationFilter, sp3: &mut SP3) {
//    match decim.dtype {
//        DecimationType::DecimByRatio(r) => {
//            let mut i = 0;
//            sp3.clock.retain(|_, _| {
//                let retained = (i % r) == 0;
//                i += 1;
//                retained
//            });
//            let mut i = 0;
//            sp3.clock_rate.retain(|_, _| {
//                let retained = (i % r) == 0;
//                i += 1;
//                retained
//            });
//            let mut i = 0;
//            sp3.position.retain(|_, _| {
//                let retained = (i % r) == 0;
//                i += 1;
//                retained
//            });
//            let mut i = 0;
//            sp3.velocities.retain(|_, _| {
//                let retained = (i % r) == 0;
//                i += 1;
//                retained
//            });
//        },
//        DecimationType::DecimByInterval(interval) => {
//            let mut last_retained = Option::<Epoch>::None;
//            sp3.clock.retain(|t, _| {
//                if let Some(last) = last_retained {
//                    let dt = *t - last;
//                    if dt >= interval {
//                        last_retained = Some(*t);
//                        true
//                    } else {
//                        false
//                    }
//                } else {
//                    last_retained = Some(*t);
//                    true // always retain 1st Epoch
//                }
//            });
//            let mut last_retained = Option::<Epoch>::None;
//            sp3.clock_rate.retain(|t, _| {
//                if let Some(last) = last_retained {
//                    let dt = *t - last;
//                    if dt >= interval {
//                        last_retained = Some(*t);
//                        true
//                    } else {
//                        false
//                    }
//                } else {
//                    last_retained = Some(*t);
//                    true // always retain 1st Epoch
//                }
//            });
//            let mut last_retained = Option::<Epoch>::None;
//            sp3.position.retain(|t, _| {
//                if let Some(last) = last_retained {
//                    let dt = *t - last;
//                    if dt >= interval {
//                        last_retained = Some(*t);
//                        true
//                    } else {
//                        false
//                    }
//                } else {
//                    last_retained = Some(*t);
//                    true // always retain 1st Epoch
//                }
//            });
//            let mut last_retained = Option::<Epoch>::None;
//            sp3.velocities.retain(|t, _| {
//                if let Some(last) = last_retained {
//                    let dt = *t - last;
//                    if dt >= interval {
//                        last_retained = Some(*t);
//                        true
//                    } else {
//                        false
//                    }
//                } else {
//                    last_retained = Some(*t);
//                    true // always retain 1st Epoch
//                }
//            });
//        },
//    }
//}

pub fn preprocess(ctx: &mut QcContext, cli: &Cli) {
    // GNSS filters
    let mut gnss_filters = Vec::<&str>::new();
    /*
     * Special teqc like filters
     * Design one filter per specs
     */
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
        let filter = Filter::from_str(filter).unwrap();
        ctx.filter_mut(&filter);
    }

    for filt_str in cli.preprocessing() {
        /*
         * Apply other -P filter specs
         */
        if let Ok(filter) = Filter::from_str(filt_str) {
            ctx.filter_mut(&filter);
            trace!("applied filter \"{}\"", filt_str);
        } else {
            error!("invalid filter description \"{}\"", filt_str);
        }
    }
}
