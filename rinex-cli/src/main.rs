//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod graph;
mod identification; // high level identification/macros
mod qc; // QC report generator // plotting operations // file operation helpers // graphical analysis

mod preprocessing;
use preprocessing::preprocess;

// mod positioning;

//use horrorshow::Template;
use rinex::prelude::*;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use cli::{Cli, Context};
// use plot::PlotContext;

//extern crate pretty_env_logger;
use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use fops::open_with_web_browser;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
    // #[error("positioning solver error")]
    // PositioningSolverError(#[from] positioning::solver::Error),
    // #[error("post processing error")]
    // PositioningPostProcError(#[from] positioning::post_process::Error),
    #[error("missing OBS RINEX")]
    MissingObservationRinex,
    #[error("missing NAV RINEX")]
    MissingNavigationRinex,
    #[error("merge ops failure")]
    MergeError(#[from] rinex::merge::Error),
    #[error("split ops failure")]
    SplitError(#[from] rinex::split::Error),
    #[error("failed to create QC report: permission denied!")]
    QcReportCreationError,
}

/*
 * Returns true if Skyplot view if feasible and allowed
fn skyplot_allowed(ctx: &RnxContext, cli: &Cli) -> bool {
    if cli.quality_check_only() || cli.positioning() {
        /*
         * Special modes: no plots allowed
         */
        return false;
    }

    let has_nav = ctx.has_navigation_data();
    let has_ref_position = ctx.ground_position().is_some() || cli.manual_position().is_some();
    if has_nav && !has_ref_position {
        info!("missing a reference position for the skyplot view.");
        info!("see rinex-cli -h : antenna positions.");
    }

    has_nav && has_ref_position
}
 */

/*
 * Returns true if NAVI plot is both feasible and allowed
fn naviplot_allowed(ctx: &RnxContext, cli: &Cli) -> bool {
    // TODO: this need to change once RnxContext gets improved
    skyplot_allowed(ctx, cli)
}
 */

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Build context defined by user
    let cli = Cli::new();
    let mut ctx = Context::from_cli(&cli)?;

    /*
     * Preprocessing
     */
    preprocess(&mut ctx.data, &cli);

    /*
     * Exclusive opmodes
     */
    match cli.matches.subcommand() {
        Some(("graph", submatches)) => {
            graph::graph_opmode(&ctx, submatches)?;
        },
        Some(("identify", submatches)) => {
            identification::dataset_identification(&ctx.data, submatches);
        },
        Some(("merge", submatches)) => {
            fops::merge(&ctx, submatches)?;
        },
        Some(("split", submatches)) => {
            fops::split(&ctx, submatches)?;
        },
        Some(("qc", submatches)) => {
            qc::qc_report(&ctx, submatches)?;
        },
        Some(("tbin", submatches)) => {
            fops::time_binning(&ctx, submatches)?;
        },
        _ => error!("no opmode specified!"),
    }
    Ok(())
    /*
     * plot combinations (if any)
    if !no_graph {
        if let Some(obs) = ctx.obs_data() {
            plot_combinations(obs, &cli, &mut plot_ctx);
        } else {
            error!("GNSS combinations requires Observation Data");
        }
    }
     */
    /*
     * skyplot
    if !no_graph {
        if skyplot_allowed(&ctx, &cli) {
            let nav = ctx.nav_data().unwrap(); // infaillble
            let ground_pos = ctx.ground_position().unwrap(); // infaillible
            plot::skyplot(nav, ground_pos, &mut plot_ctx);
            info!("skyplot view generated");
        } else if !no_graph {
            info!("skyplot view is not feasible");
        }
    }
     */
    // if positioning {
    //     let results = positioning::solver(&mut ctx, &cli)?;
    //     positioning::post_process(workspace, &cli, &ctx, results)?;
    // }
} // main
