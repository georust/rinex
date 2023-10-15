//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
pub mod fops; // file operation helpers
mod identification; // high level identification/macros
mod plot; // plotting operations
mod rtk_postproc; // rtk results post processing

mod preprocessing;
use preprocessing::preprocess;

//use horrorshow::Template;
use rinex::{
    merge::Merge,
    observation::{Combine, Dcb, IonoDelay}, //Mp},
    prelude::*,
    split::Split,
};

extern crate gnss_rtk as rtk;
use rtk::prelude::{Solver, SolverError, SolverEstimate, SolverType};

use rinex_qc::*;

use cli::Cli;
use identification::rinex_identification;
use plot::PlotContext;

//extern crate pretty_env_logger;
use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use fops::open_with_web_browser;
use rtk_postproc::rtk_postproc;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
    #[error("rtk post proc error")]
    RTKPostError(#[from] rtk_postproc::Error),
}

/*
 * Workspace location is fixed to rinex-cli/product/$primary
 * at the moment
 */
pub fn workspace_path(ctx: &RnxContext) -> PathBuf {
    let primary_stem: &str = ctx
        .primary_paths()
        .first()
        .expect("failed to determine Workspace")
        .file_stem()
        .expect("failed to determine Workspace")
        .to_str()
        .expect("failed to determine Workspace");
    /*
     * In case $FILENAME.RNX.gz gz compressed, we extract "$FILENAME".
     * Can use .file_name() once https://github.com/rust-lang/rust/issues/86319  is stabilized
     */
    let primary_stem: Vec<&str> = primary_stem.split('.').collect();

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("workspace")
        .join(primary_stem[0])
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
 * Loads augmentation directories recursively
 */
fn load_recursive_augmentation(file: &str, ctx: &mut RnxContext, base_dir: &str, max_depth: usize) {
    let walkdir = WalkDir::new(base_dir).max_depth(max_depth);
    for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
        if !entry.path().is_dir() {
            let fullpath = entry.path().to_string_lossy().to_string();
            if ctx.load(&fullpath).is_err() {
                warn!("failed to load --{} \"{}\"", file, fullpath);
            }
        }
    }
}

/*
 * Creates File/Data context defined by user.
 * Regroups all provided files/folders,
 */
fn build_context(cli: &Cli) -> RnxContext {
    let path = cli.input_path();
    let pathbf = Path::new(path).to_path_buf();
    let ctx = RnxContext::new(pathbf);
    if ctx.is_err() {
        panic!(
            "failed to load desired context \"{}\", : {:?}",
            path,
            ctx.err().unwrap()
        );
    }
    let mut ctx = ctx.unwrap();
    /*
     * load possible augmentations
     */
    for path in cli.navigation_paths() {
        let path = Path::new(path);
        let fullpath = path.to_string_lossy().to_string();
        if path.is_dir() {
            load_recursive_augmentation("nav", &mut ctx, &fullpath, 5);
        } else if ctx.load(&fullpath).is_err() {
            warn!("failed to load --nav \"{}\"", fullpath);
        }
    }
    for path in cli.atx_paths() {
        let path = Path::new(path);
        let fullpath = path.to_string_lossy().to_string();
        if path.is_dir() {
            load_recursive_augmentation("atx", &mut ctx, &fullpath, 5);
        } else if ctx.load(&fullpath).is_err() {
            warn!("failed to load --atx \"{}\"", fullpath);
        }
    }
    for path in cli.sp3_paths() {
        let path = Path::new(path);
        let fullpath = path.to_string_lossy().to_string();
        if path.is_dir() {
            load_recursive_augmentation("sp3", &mut ctx, &fullpath, 5);
        } else if ctx.load(&fullpath).is_err() {
            warn!("failed to load --sp3 \"{}\"", fullpath);
        }
    }
    ctx
}

/*
 * Returns true if Skyplot view if feasible
 */
fn skyplot_allowed(ctx: &RnxContext, cli: &Cli) -> bool {
    if cli.quality_check_only() || cli.rtk_only() {
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

    let rtk_only = cli.rtk_only();
    let rtk = cli.rtk() || rtk_only;

    if cli.multipath() {
        warn!("--mp analysis not available yet");
    }

    // Initiate plot context
    let mut plot_ctx = PlotContext::new();

    // Initiate QC parameters
    let mut qc_opts = cli.qc_config();

    // Build context
    let mut ctx = build_context(&cli);

    // Position solver
    let mut solver = Solver::from(&ctx);
    if let Ok(ref mut solver) = solver {
        info!(
            "provided context is compatible with {} position solver",
            solver.solver
        );
        // custom config ? apply it
        if let Some(cfg) = cli.rtk_config() {
            solver.cfg = cfg.clone();
        }
        if !rtk {
            warn!("position solver currently turned off");
        } else {
            if cli.forced_spp() {
                warn!("forced method to spp");
                solver.solver = SolverType::SPP;
            }
            // print config to be used
            info!("{:#?}", solver.cfg);

            // print more infos
            if ctx.sp3_data().is_none() {
                error!("--rtk does not work without SP3 at the moment");
            }
        }
    } else {
        warn!("context is not sufficient or not compatible with --rtk");
    }

    // Workspace
    let workspace = workspace_path(&ctx);
    info!("workspace is \"{}\"", workspace.to_string_lossy());
    create_workspace(workspace.clone());

    /*
     * Print more info on special primary data cases
     */
    if ctx.primary_data().is_meteo_rinex() {
        info!("meteo special primary data");
    } else if ctx.primary_data().is_ionex() {
        info!("ionex special primary data");
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
     * SV per Epoch analysis requested
     */
    if cli.sv_epoch() && !no_graph {
        info!("sv/epoch analysis");
        analysis::sv_epoch(&ctx, &mut plot_ctx);
    }
    /*
     * Epoch histogram analysis
     */
    if cli.sampling_histogram() && !no_graph {
        info!("sample rate histogram analysis");
        analysis::sampling::histogram(&ctx, &mut plot_ctx);
    }
    /*
     * DCB analysis requested
     */
    if cli.dcb() && !no_graph {
        let data = ctx
            .primary_data()
            .observation_phase_align_origin()
            .observation_phase_carrier_cycles()
            .dcb();
        plot::plot_gnss_dcb(
            &mut plot_ctx,
            "Differential Code Biases",
            "DBCs [n.a]",
            &data,
        );
        info!("--dcb analysis");
    }
    /*
     * Code Multipath analysis
     */
    if cli.multipath() && !no_graph {
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
    }
    /*
     * [GF] recombination visualization requested
     */
    if cli.gf_recombination() && !no_graph {
        let data = ctx
            .primary_data()
            .observation_phase_align_origin()
            .geo_free();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Geometry Free signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--gf recombination");
    }
    /*
     * Ionospheric Delay Detector (graph)
     */
    if cli.iono_detector() && !no_graph {
        let data = ctx.primary_data().iono_delay(Duration::from_seconds(360.0));
        plot::plot_iono_detector(&mut plot_ctx, &data);
        info!("--iono detector");
    }
    /*
     * [WL] recombination
     */
    if cli.wl_recombination() && !no_graph {
        let data = ctx.primary_data().wide_lane();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Wide Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--wl recombination");
    }
    /*
     * [NL] recombination
     */
    if cli.nl_recombination() && !no_graph {
        let data = ctx.primary_data().narrow_lane();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Narrow Lane signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--nl recombination");
    }
    /*
     * [MW] recombination
     */
    if cli.mw_recombination() && !no_graph {
        let data = ctx.primary_data().melbourne_wubbena();
        plot::plot_gnss_recombination(
            &mut plot_ctx,
            "Melbourne-Wübbena signal combination",
            "Meters of Li-Lj delay",
            &data,
        );
        info!("--mw recombination");
    }
    /*
     * MERGE
     */
    if let Some(rinex_b) = cli.to_merge() {
        let new_rinex = ctx
            .primary_data()
            .merge(&rinex_b)
            .expect("failed to merge both files");

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
        let (rnx_a, rnx_b) = ctx
            .primary_data()
            .split(epoch)
            .expect("failed to split primary rinex file");

        let file_stem = ctx
            .primary_paths()
            .first()
            .expect("failed to determine file prefix")
            .file_stem()
            .expect("failed to determine file prefix")
            .to_str()
            .expect("failed to determine file prefix");

        let file_suffix = rnx_a
            .first_epoch()
            .expect("failed to determine file suffix")
            .to_string();

        let path = format!(
            "{}/{}-{}.txt",
            workspace.to_string_lossy(),
            file_stem,
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
            file_stem,
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
    if skyplot_allowed(&ctx, &cli) && !no_graph {
        plot::skyplot(&ctx, &mut plot_ctx);
        info!("skyplot view generated");
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
    if !qc_only && !rtk_only && !no_graph {
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

    if !rtk {
        return Ok(());
    }

    if let Ok(ref mut solver) = solver {
        /* init */
        match solver.init(&mut ctx) {
            Err(e) => panic!("failed to initialize rtk solver - {}", e),
            Ok(_) => info!("entering rtk mode"),
        }

        // position solver feasible & deployed
        let mut solving = true;
        let mut results: HashMap<Epoch, SolverEstimate> = HashMap::new();

        while solving {
            match solver.run(&mut ctx) {
                Ok((t, estimate)) => {
                    trace!("{:?}", t);
                    results.insert(t, estimate);
                },
                Err(SolverError::NoSv(t)) => info!("no SV elected @{}", t),
                Err(SolverError::LessThan4Sv(t)) => info!("less than 4 SV @{}", t),
                Err(SolverError::SolvingError(t)) => {
                    error!("failed to invert navigation matrix @ {}", t)
                },
                Err(SolverError::EpochDetermination(_)) => {
                    solving = false; // abort
                },
                Err(e) => panic!("fatal error {:?}", e),
            }
        }
        rtk_postproc(workspace, &cli, &ctx, results)?;
    }
    Ok(())
} // main
