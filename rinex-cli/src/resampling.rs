use crate::{
    Cli,
    Context,
    parser::{parse_duration, parse_epoch},
};
use log::{info, error, warn};
use rinex::{processing::Decimation, *};

/// Efficient RINEX content decimation
pub fn resampling(ctx: &mut Context, cli: &Cli) {
    let resampling_ops = cli.resampling_ops();
    for (op, args) in resampling_ops.iter() {
        if op.eq(&"time-window") {
            let items: Vec<&str> = args.split(" ").collect();
            if items.len() == 2 {
                // date description
                if let Ok(start) = parse_epoch(items[0].trim()) {
                    if let Ok(end) = parse_epoch(items[1].trim()) {
                        ctx.primary_rinex.time_window_mut(start, end);
                        trace!("applied time window: {} - {}", start, end);
                    } else {
                        error!(
                            "failed to parse epoch from \"{}\" description",
                            items[1],
                        );
                        warn!("time window not applied");
                    }
                } else {
                    error!(
                        "failed to parse epoch from \"{}\" description",
                        items[0],
                    );
                    warn!("time window not applied");
                }

            } else if items.len() == 4 {
                //datetime description
                let mut start_str = items[0].trim().to_owned();
                start_str.push_str(" ");
                start_str.push_str(items[1].trim());

                if let Ok(start) = parse_epoch(&start_str) {
                    let mut end_str = items[2].trim().to_owned();
                    end_str.push_str(" ");
                    end_str.push_str(items[3].trim());

                    if let Ok(end) = parse_epoch(&end_str) {
                        ctx.primary_rinex.time_window_mut(start, end);
                        if let Some(ref mut nav) = ctx.nav_rinex {
                            nav.time_window_mut(start, end);
                        }
                        trace!("applied time window: {} - {}", start, end);
                    }
                } else {
                    error!(
                        "failed to parse epoch from \"{}\" description",
                        start_str
                    );
                    warn!("time window not applied");
                }
            } else {
                error!("invalid time window description");
            }

        } else if op.eq(&"resample-interval") {
            if let Ok(duration) = parse_duration(args.trim()) {
                ctx.primary_rinex.decim_by_interval_mut(duration);
                if let Some(ref mut nav) = ctx.nav_rinex {
                    nav.decim_by_interval_mut(duration);
                }
                trace!("record decimation - ok");
            } else {
                error!("failed to parse duration from \"{}\"", args);
                warn!("record decimation not effective");
            }

        } else if op.eq(&"resample-ratio") {
            if let Ok(ratio) = u32::from_str_radix(args.trim(), 10) {
                ctx.primary_rinex.decim_by_ratio_mut(ratio);
                if let Some(ref mut nav) = ctx.nav_rinex {
                    nav.decim_by_ratio_mut(ratio);
                }
                trace!("record decimation - ok");
            } else {
                error!("failed to parse decimation ratio from \"{}\"", args);
                warn!("record decimation not effective");
            }
        }
    }
}
