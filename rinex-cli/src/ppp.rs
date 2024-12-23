use crate::cli::Cli;
use clap::ArgMatches;

use rinex_qc::prelude::{QcContext, RTKConfig, RTKMethod};

pub fn ppp(ctx: &QcContext, cli: &Cli, args: &ArgMatches) {
    let mut cfg = RTKConfig::default();
    cfg.method = RTKMethod::SPP;

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
