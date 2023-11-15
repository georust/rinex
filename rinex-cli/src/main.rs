//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
pub mod fops; // file operation helpers
mod identification; // high level identification/macros
mod plot; // plotting operations

mod preprocessing;
use preprocessing::preprocess;

mod positioning;

//use horrorshow::Template;
use rinex::{
    merge::Merge,
    observation::{Combine, Dcb, IonoDelay}, //Mp},
    prelude::*,
    split::Split,
};

use rinex_qc::*;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use cli::Cli;
use identification::rinex_identification;
use plot::PlotContext;

//extern crate pretty_env_logger;
use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use fops::open_with_web_browser;
use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
    #[error("positioning solver error")]
    PositioningSolverError(#[from] positioning::solver::Error),
    #[error("post processing error")]
    PositioningPostProcError(#[from] positioning::post_process::Error),
}

/*
 * Utility : determines  the file stem of most major RINEX file in the context
 */
pub(crate) fn context_stem(ctx: &RnxContext) -> String {
    let ctx_major_stem: &str = ctx
        .rinex_path()
        .expect("failed to determine a context name")
        .file_stem()
        .expect("failed to determine a context name")
        .to_str()
        .expect("failed to determine a context name");
    /*
     * In case $FILENAME.RNX.gz gz compressed, we extract "$FILENAME".
     * Can use .file_name() once https://github.com/rust-lang/rust/issues/86319  is stabilized
     */
    let primary_stem: Vec<&str> = ctx_major_stem.split('.').collect();
    primary_stem[0].to_string()
}

/*
 * Workspace location is fixed to rinex-cli/product/$primary
 * at the moment
 */
pub fn workspace_path(ctx: &RnxContext) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("workspace")
        .join(&context_stem(ctx))
}

/*
 * Helper to create the workspace, ie.: where all reports
 * get generated and saved.
 */
pub fn create_workspace(path: PathBuf) {
    std::fs::create_dir_all(&path).unwrap_or_else(|_| {
        panic!(
            "failed to create Workspace \"{}\": permission denied!",
            path.to_string_lossy()
        )
    });
}

use walkdir::WalkDir;

/*
 * Creates File/Data context defined by user.
 * Regroups all provided files/folders,
 */
fn build_context(cli: &Cli) -> RnxContext {
    let mut ctx = RnxContext::default();
    /* load all directories recursively, one by one */
    for dir in cli.input_directories() {
        let walkdir = WalkDir::new(dir).max_depth(5);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let filepath = entry.path().to_string_lossy().to_string();
                let ret = ctx.load(&filepath);
                if ret.is_err() {
                    warn!("failed to load \"{}\": {}", filepath, ret.err().unwrap());
                }
            }
        }
    }
    // load individual files, if any
    for filepath in cli.input_files() {
        let ret = ctx.load(filepath);
        if ret.is_err() {
            warn!("failed to load \"{}\": {}", filepath, ret.err().unwrap());
        }
    }
    ctx
}

/*
 * Returns true if Skyplot view if feasible and allowed
 */
fn skyplot_allowed(ctx: &RnxContext, cli: &Cli) -> bool {
    if cli.quality_check_only() || cli.positioning_only() {
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

/*
 * Returns true if NAVI plot is both feasible and allowed
 */
fn naviplot_allowed(ctx: &RnxContext, cli: &Cli) -> bool {
    // TODO: this need to change once RnxContext gets improved
    skyplot_allowed(ctx, cli)
}

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Cli
    let cli = Cli::new();
    let quiet = cli.quiet();
    let no_graph = cli.no_graph();

    let qc_only = cli.quality_check_only();
    let qc = cli.quality_check() || qc_only;

    let positioning_only = cli.positioning_only();
    let positioning = cli.spp() || cli.ppp() || positioning_only;

    if !positioning {
        warn!("position solver currently turned off");
    }

    if cli.multipath() {
        warn!("--mp analysis not available yet");
    }

    // Initiate plot context
    let mut plot_ctx = PlotContext::new();

    // Initiate QC parameters
    let mut qc_opts = cli.qc_config();

    // Build context
    let mut ctx = build_context(&cli);

    // Workspace
    let workspace = workspace_path(&ctx);
    info!("workspace is \"{}\"", workspace.to_string_lossy());
    create_workspace(workspace.clone());

    /*
     * Print more info on provided context
     */
    if ctx.obs_data().is_some() {
        info!("observation data loaded");
    }
    if ctx.nav_data().is_some() {
        info!("brdc navigation data loaded");
    }
    if ctx.sp3_data().is_some() {
        info!("sp3 data loaded");
    }
    if ctx.meteo_data().is_some() {
        info!("meteo data loaded");
    }
    if ctx.ionex_data().is_some() {
        info!("ionex data loaded");
    }

    /*
     * Emphasize which reference position is to be used.
     * This will help user make sure everything is correct.
     * [+] Cli: always superceeds
     * [+] then QC config file is prefered (advanced usage)
     * [+] eventually we rely on the context pool.
     * Missing ref. position may restrict possible operations.
     */
    if let Some(pos) = cli.manual_position() {
        let (lat, lon, _) = pos.to_geodetic();
        info!(
            "using manually defined reference position {} (lat={:.5}°, lon={:.5}°)",
            pos, lat, lon
        );
    } else if let Some(pos) = ctx.ground_position() {
        let (lat, lon, _) = pos.to_geodetic();
        info!(
            "using reference position {} (lat={:.5}°, lon={:.5}°)",
            pos, lat, lon
        );
    } else {
        info!("no reference position given or identified");
    }
    /*
     * Preprocessing
     */
    preprocess(&mut ctx, &cli);
    /*
     * Basic file identification
     */
    if cli.identification() {
        rinex_identification(&ctx, &cli);
        return Ok(()); // not proceeding further, in this mode
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx
                .observation_phase_align_origin()
                .observation_phase_carrier_cycles()
                .dcb();
            plot::plot_gnss_dcb(
                &mut plot_ctx,
                "Differential Code Biases",
                "DBCs [n.a]",
                &data,
            );
            info!("dcb analysis");
        } else {
            error!("dcb is not feasible");
        }
    }
    /*
     * Code Multipath analysis
     */
    if cli.multipath() && !no_graph {
        if let Some(_rnx) = ctx.obs_data() {
            //let data = ctx
            //    .primary_data()
            //    .observation_phase_align_origin()
            //    .observation_phase_carrier_cycles()
            //    .mp();
            //plot::plot_gnss_dcb(
            //    &mut plot_ctx,
            //    "Code Multipath Biases",
            //    "Meters of delay",
            //    &data,
            //);
        } else {
            error!("mp is not feasible");
        }
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx.observation_phase_align_origin().geo_free();
            plot::plot_gnss_recombination(
                &mut plot_ctx,
                "Geometry Free signal combination",
                "Meters of Li-Lj delay",
                &data,
            );
            info!("gf recombination");
        } else {
            error!("gf is not feasible");
        }
    }
    /*
     * Ionospheric Delay Detector (graph)
     */
    if cli.iono_detector() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx.iono_delay(Duration::from_seconds(360.0));
            plot::plot_iono_detector(&mut plot_ctx, &data);
            info!("iono detector");
        } else {
            error!("iono is not feasible");
        }
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx.wide_lane();
            plot::plot_gnss_recombination(
                &mut plot_ctx,
                "Wide Lane signal combination",
                "Meters of Li-Lj delay",
                &data,
            );
            info!("wl recombination");
        } else {
            error!("wl is not feasible");
        }
    }
    /*
     * [NL] recombination
     */
    if cli.nl_recombination() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx.narrow_lane();
            plot::plot_gnss_recombination(
                &mut plot_ctx,
                "Narrow Lane signal combination",
                "Meters of Li-Lj delay",
                &data,
            );
            info!("nl recombination");
        } else {
            error!("nl is not feasible");
        }
    }
    /*
     * [MW] recombination
     */
    if cli.mw_recombination() && !no_graph {
        if let Some(rnx) = ctx.obs_data() {
            let data = rnx.melbourne_wubbena();
            plot::plot_gnss_recombination(
                &mut plot_ctx,
                "Melbourne-Wübbena signal combination",
                "Meters of Li-Lj delay",
                &data,
            );
            info!("mw recombination");
        } else {
            error!("mw is not feasible");
        }
    }
    /*
     * MERGE
     */
    if let Some(rinex_b) = cli.to_merge() {
        let rinex = ctx.rinex_data().expect("undefined RINEX data");

        let new_rinex = rinex.merge(&rinex_b).expect("failed to merge both files");

        let filename = match cli.output_path() {
            Some(path) => path.clone(),
            None => String::from("merged.rnx"),
        };

        let path = workspace.clone().join(&filename);

        let path = path
            .as_path()
            .to_str()
            .expect("failed to generate merged file");

        // generate new file
        new_rinex
            .to_file(path)
            .expect("failed to generate merged file");

        info!("\"{}\" has been generated", &path);
        return Ok(());
    }
    /*
     * SPLIT
     */
    if let Some(epoch) = cli.split() {
        let rinex = ctx.rinex_data().expect("undefined RINEX data");

        let (rnx_a, rnx_b) = rinex
            .split(epoch)
            .expect("failed to split primary rinex file");

        let file_suffix = rnx_a
            .first_epoch()
            .expect("failed to determine file suffix")
            .to_string();

        let path = format!(
            "{}/{}-{}.txt",
            workspace.to_string_lossy(),
            context_stem(&ctx),
            file_suffix
        );

        rnx_a
            .to_file(&path)
            .unwrap_or_else(|_| panic!("failed to generate splitted file \"{}\"", path));

        let file_suffix = rnx_b
            .first_epoch()
            .expect("failed to determine file suffix")
            .to_string();

        let path = format!(
            "{}/{}-{}.txt",
            workspace.to_string_lossy(),
            context_stem(&ctx),
            file_suffix
        );

        rnx_b
            .to_file(&path)
            .unwrap_or_else(|_| panic!("failed to generate splitted file \"{}\"", path));

        // [*] stop here, special mode: no further analysis allowed
        return Ok(());
    }
    /*
     * skyplot
     */
    if !no_graph {
        if skyplot_allowed(&ctx, &cli) {
            let nav = ctx.nav_data().unwrap(); // infaillble
            let ground_pos = ctx.ground_position().unwrap(); // infaillible
            plot::skyplot(nav, ground_pos, &mut plot_ctx);
            info!("skyplot view generated");
        } else {
            if !no_graph {
                info!("skyplot view is not feasible");
            }
        }
    }
    /*
     * 3D NAVI plot
     */
    if naviplot_allowed(&ctx, &cli) && !no_graph {
        plot::naviplot(&ctx, &mut plot_ctx);
        info!("navi plot generated");
    }
    /*
     * CS Detector
     */
    if cli.cs_graph() && !no_graph {
        info!("cs detector");
        //let mut detector = CsDetector::default();
        //let cs = detector.cs_detection(&ctx.primary_rinex);
    }
    /*
     * Record analysis / visualization
     * analysis depends on the provided record type
     */
    if !qc_only && !positioning_only && !no_graph {
        info!("entering record analysis");
        plot::plot_record(&ctx, &mut plot_ctx);

        /*
         * Render Graphs (HTML)
         */
        let html_path = workspace_path(&ctx).join("graphs.html");
        let html_path = html_path.to_str().unwrap();

        let mut html_fd = std::fs::File::create(html_path)
            .unwrap_or_else(|_| panic!("failed to create \"{}\"", &html_path));
        write!(html_fd, "{}", plot_ctx.to_html()).expect("failed to render graphs");

        info!("graphs rendered in $WORKSPACE/graphs.html");
        if !quiet {
            open_with_web_browser(html_path);
        }
    }
    /*
     * QC Mode
     */
    if qc {
        info!("entering qc mode");
        /*
         * QC Config / versus command line
         * let the possibility to define some parameters
         * from the command line, instead of the config file.
         */
        if qc_opts.ground_position.is_none() {
            // config did not specify it
            if let Some(pos) = cli.manual_position() {
                // manually passed
                qc_opts = qc_opts.with_ground_position_ecef(pos.to_ecef_wgs84());
            }
        }

        /*
         * Print some info about current setup & parameters, prior analysis.
         */
        info!("Classification method : {:?}", qc_opts.classification);
        info!("Reference position    : {:?}", qc_opts.ground_position);
        info!("Minimal SNR           : {:?}", qc_opts.min_snr_db);
        info!("Elevation mask        : {:?}", qc_opts.elev_mask);
        info!("Sampling gap tolerance: {:?}", qc_opts.gap_tolerance);

        let html_report = QcReport::html(&ctx, qc_opts);

        let report_path = workspace.join("report.html");
        let mut report_fd = std::fs::File::create(&report_path).unwrap_or_else(|_| {
            panic!(
                "failed to create report \"{}\" : permission denied",
                report_path.to_string_lossy()
            )
        });

        write!(report_fd, "{}", html_report).expect("failed to generate QC summary report");

        info!("qc report $WORSPACE/report.html has been generated");
        if !quiet {
            open_with_web_browser(&report_path.to_string_lossy());
        }
    }

    if positioning {
        let results = positioning::solver(&mut ctx, &cli)?;
        positioning::post_process(workspace, &cli, &ctx, results)?;
    }

    Ok(())
} // main
