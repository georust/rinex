use crate::cli::Cli;
use clap::ArgMatches;

use rinex::prelude::Duration;

use rinex_qc::prelude::{
    PVTSolutionType, QcContext, RTKConfig, RTKMethod, BIPM_CGGTTS_TRACKING_DURATION_SECONDS,
};

/// ppp opmode
pub fn ppp(ctx: &QcContext, cli: &Cli, opts: &ArgMatches) {
    let mut cfg = RTKConfig::default();

    if opts.get_flag("cggtts") {
        cfg.sol_type = PVTSolutionType::TimeOnly;

        return ppp_cggtts(cfg, ctx, cli, opts);
    }

    for meta in ctx.observations_meta() {
        let meta_rx_orbit = ctx.meta_rx_orbit(meta);

        let mut solver = ctx
            .nav_pvt_solver(cfg.clone(), meta, meta_rx_orbit)
            .unwrap();

        while let Some(solution) = solver.next() {
            println!("solution: {:?}", solution);
        }
    }
}

/// PPP with special --cggtts option
pub fn ppp_cggtts(cfg: RTKConfig, ctx: &QcContext, cli: &Cli, opts: &ArgMatches) {
    let tracking_dt = match opts.get_one::<Duration>("tracking") {
        Some(dt) => {
            info!("using custom tracking duration: {}", *dt);
            *dt
        },
        None => {
            let dt = Duration::from_seconds(BIPM_CGGTTS_TRACKING_DURATION_SECONDS.into());
            info!("using BIPM tracking duration: {}", dt);
            dt
        },
    };

    for meta in ctx.observations_meta() {
        let mut solver = ctx
            .nav_cggtts_solver(cfg.clone(), meta, None, tracking_dt)
            .unwrap();

        while let Some(track) = solver.next() {
            println!("track: {:?}", track);
        }
    }
}
