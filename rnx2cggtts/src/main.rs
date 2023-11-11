#[macro_use]
extern crate log;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use rinex::prelude::*;

use cggtts::prelude::*;
use cggtts::Coordinates;

use cli::Cli;

use env_logger::{Builder, Target};

// use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use thiserror::Error;

mod cli; // command line interface
pub mod fops; // file operation helpers

mod preprocessing;
use preprocessing::preprocess;

mod solver;

use std::fs::File;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
    #[error("failed to format cggtts")]
    CggttsWriteError(#[from] std::io::Error),
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
        .join("..")
        .join("rinex-cli")
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

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Cli
    let cli = Cli::new();

    // Build context
    let mut ctx = build_context(&cli);

    // Build position solver
    if ctx.sp3_data().is_none() {
        error!("SP3 must unfortunately be provided at the moment");
        return Ok(());
    }

    // Workspace
    let workspace = workspace_path(&ctx);
    info!("workspace is \"{}\"", workspace.to_string_lossy());
    create_workspace(workspace.clone());

    /*
     * Verify provided context and feasibility
     */
    if ctx.obs_data().is_none() {
        panic!("rnx2cggtts requires Observation Data to be provided!");
    }
    if ctx.nav_data().is_none() {
        panic!("rnx2cggtts requires BRDC Navigation Data to be provided!");
    }
    if ctx.sp3_data().is_none() {
        panic!("rnx2cggtts requires SP3 Data to be provided!");
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
     * [+] eventually we rely on the context pool.
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
     * System delay(s) to be compensated
     */
    if let Some(rf_delay) = cli.rf_delay() {
        for (code, delay_ns) in rf_delay {
            // solver.cfg.internal_delay.insert(code.clone(), delay_ns);
            info!("RF delay: {} : {} [ns]", code.clone(), delay_ns);
        }
    }
    if let Some(delay_ns) = cli.reference_time_delay() {
        // solver.cfg.time_ref_delay = Some(delay_ns);
        info!("REFERENCE delay: {} [ns]", delay_ns);
    }
    /*
     * Preprocessing
     */
    preprocess(&mut ctx, &cli);

    /*
     * Form CGGTTS
     */
    let obs_data = ctx.obs_data().unwrap();

    let rcvr = match &obs_data.header.rcvr {
        Some(rcvr) => Rcvr {
            manufacturer: String::from("XX"),
            model: rcvr.model.clone(),
            serial_number: rcvr.sn.clone(),
            year: 0,
            release: rcvr.firmware.clone(),
        },
        None => Rcvr::default(),
    };

    let apc = Coordinates {
        x: 0.0_f64,
        y: 0.0_f64,
        z: 0.0_f64,
    };

    let station: String = match cli.custom_station() {
        Some(station) => station.to_string(),
        None => {
            let stem = context_stem(&ctx);
            if let Some(index) = stem.find('_') {
                stem[..index].to_string()
            } else {
                String::from("LAB")
            }
        },
    };

    let mut cggtts = CGGTTS::default()
        .station(&station)
        .nb_channels(1)
        .receiver(rcvr)
        //.ims(ims) // TODO
        .apc_coordinates(apc)
        .reference_time(cli.reference_time())
        .reference_frame("WGS84")
        .comments(&format!("rnx2cggtts v{}", env!("CARGO_PKG_VERSION")));

    /*
     * Form TRACKS
     */
    let tracks = solver::resolve(&mut ctx, &cli);

    if let Ok(tracks) = tracks {
        for track in tracks {
            cggtts.tracks.push(track);
        }
    }

    /*
     * Create file
     */
    let filename = match cli.custom_filename() {
        Some(filename) => filename.to_string(),
        None => cggtts.filename(),
    };

    let mut fd = File::create(&filename)?;
    write!(fd, "{}", cggtts)?;
    info!("{} has been generated", filename);
    Ok(())
}
