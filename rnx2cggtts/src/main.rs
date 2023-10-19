mod cli; // command line interface
mod plot; // plotting operations
pub mod fops; // file operation helpers

mod preprocessing;
use preprocessing::preprocess;

//use horrorshow::Template;
use rinex::prelude::*;

extern crate gnss_rtk as rtk;
use rtk::prelude::{Solver, SolverError, SolverEstimate, SolverType};

use cggtts::{Cggtts, Track as CggttsTrack};

use cli::Cli;
use plot::PlotContext;

//extern crate pretty_env_logger;
use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use fops::open_with_web_browser;
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
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

/*
 * Creates File/Data context defined by user.
 * Regroups all provided files/folders,
 */
fn build_context(cli: &Cli) -> RnxContext {
    // Load base dir (if any)
    if let Some(base_dir) = cli.input_base_dir() {
        let ctx = RnxContext::new(base_dir.into());
        if ctx.is_err() {
            panic!(
                "failed to load desired context \"{}\", : {:?}",
                base_dir,
                ctx.err().unwrap()
            );
        }
        let mut ctx = ctx.unwrap();
        // Append individual files, if any
        for filepath in cli.input_files() {
            if ctx.load(filepath).is_err() {
                warn!("failed to load \"{}\"", filepath);
            }
        }
        ctx
    } else {
        // load individual files, if any
        let mut ctx = RnxContext::default();
        for filepath in cli.input_files() {
            if ctx.load(filepath).is_err() {
                warn!("failed to load \"{}\"", filepath);
            }
        }
        ctx
    }
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
    let use_graph = cli.graph();

    // Init. plot context
    let mut plot_ctx = PlotContext::new();

    // Build context
    let mut ctx = build_context(&cli);

    // Build position solver
    let mut solver = Solver::from(&ctx)
        .expect("provided context is not compatible with a position solving method");
    
    if ctx.sp3_data().is_none() {
        error!("SP3 must unfortunately be provided at the moment");
        return Ok(());
    }

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

    /* init solver */
    match solver.init(&mut ctx) {
        Err(e) => panic!("failed to initialize rtk solver - {}", e),
        Ok(_) => info!("entering rtk mode"),
    }

    // RUN
    let mut track = CggttsTrack::new();
    let mut tracks: Vec<CggttsTrack> = Vec::new();

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
    /* 
     * Form CGGTTS
     */
    let cggtts = Cggtts {
        rev_date:,
        lab:,
        rcvr:,
        nb_channels:,
        ims:,
        time_reference:,
        reference_frame: ,
        coordinates: ,
        comments: Some(vec![format!("Generated with rnx2cggtts {}", env!("CARGO_ENV_PACKAGE"))),
        delay: ,
        tracks,
    };
    Ok(())
}
