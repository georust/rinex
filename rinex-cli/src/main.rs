//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod identification; // high level identification/macros
mod plot;
mod qc; // QC report generator // plotting operations // file operation helpers

mod preprocessing;
use preprocessing::preprocess;

// mod positioning;

//use horrorshow::Template;
use rinex::{
    observation::{Combination, Combine},
    prelude::*,
};

use rinex_qc::*;

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

/*
 * Plots requested combinations
fn plot_combinations(obs: &Rinex, cli: &Cli, plot_ctx: &mut PlotContext) {
    //if cli.dcb() {
    //    let data = obs.dcb();
    //    plot::plot_gnss_dcb(
    //        plot_ctx,
    //        "Differential Code Biases",
    //        "Differential Code Bias [s]",
    //        &data,
    //    );
    //    info!("dcb analysis");
    //}
    if cli.multipath() {
        let data = obs.code_multipath();
        plot::plot_gnss_dcb_mp(&data, plot_ctx, "Code Multipath", "Meters of delay");
        info!("code multipath analysis");
    }
    if cli.if_combination() {
        let data = obs.combine(Combination::IonosphereFree);
        plot::plot_gnss_combination(
            &data,
            plot_ctx,
            "Ionosphere Free combination",
            "Meters of delay",
        );
        info!("iono free combination");
    }
    if cli.gf_combination() {
        let data = obs.combine(Combination::GeometryFree);
        plot::plot_gnss_combination(
            &data,
            plot_ctx,
            "Geometry Free combination",
            "Meters of delay",
        );
        info!("geo free combination");
    }
    if cli.wl_combination() {
        let data = obs.combine(Combination::WideLane);
        plot::plot_gnss_combination(&data, plot_ctx, "Wide Lane combination", "Meters of delay");
        info!("wide lane combination");
    }
    if cli.nl_combination() {
        let data = obs.combine(Combination::NarrowLane);
        plot::plot_gnss_combination(
            &data,
            plot_ctx,
            "Narrow Lane combination",
            "Meters of delay",
        );
        info!("wide lane combination");
    }
    if cli.mw_combination() {
        let data = obs.combine(Combination::MelbourneWubbena);
        plot::plot_gnss_combination(
            &data,
            plot_ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
        );
        info!("melbourne-wubbena combination");
    }
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
    /*
     * 3D NAVI plot
    if naviplot_allowed(&ctx, &cli) && !no_graph {
        plot::naviplot(&ctx, &mut plot_ctx);
        info!("navi plot generated");
    }
     */
    /*
     * Record analysis / visualization
     * analysis depends on the provided record type
    if !qc_only && !positioning && !no_graph {
        info!("entering record analysis");
        plot::plot_record(&ctx, &mut plot_ctx);

        /*
         * Render Graphs (HTML)
         */
        let html_path = workspace_path(&ctx, &cli).join("graphs.html");
        let html_path = html_path.to_str().unwrap();

        let mut html_fd = std::fs::File::create(html_path)
            .unwrap_or_else(|_| panic!("failed to create \"{}\"", &html_path));
        write!(html_fd, "{}", plot_ctx.to_html()).expect("failed to render graphs");

        info!("graphs rendered in $WORKSPACE/graphs.html");
        if !quiet {
            open_with_web_browser(html_path);
        }
    }
     */
    // if positioning {
    //     let results = positioning::solver(&mut ctx, &cli)?;
    //     positioning::post_process(workspace, &cli, &ctx, results)?;
    // }
} // main
