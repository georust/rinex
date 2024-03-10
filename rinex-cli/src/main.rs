//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

//mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod graph;
mod identification; // high level identification/macros
mod positioning;
mod qc; // QC report generator // plotting operations // file operation helpers // graphical analysis // positioning + CGGTTS opmode

mod preprocessing;
use preprocessing::preprocess;

use rinex::prelude::RnxContext;

use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use rinex::prelude::Rinex;
use sp3::prelude::SP3;

use cli::{Cli, Context};

use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};

use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rinex error")]
    RinexError(#[from] rinex::Error),
    #[error("missing OBS RINEX")]
    MissingObservationRinex,
    #[error("missing (BRDC) NAV RINEX")]
    MissingNavigationRinex,
    #[error("merge ops failure")]
    MergeError(#[from] rinex::merge::Error),
    #[error("split ops failure")]
    SplitError(#[from] rinex::split::Error),
    #[error("failed to create QC report: permission denied!")]
    QcReportCreationError,
    #[error("positioning solver error")]
    PositioningSolverError(#[from] positioning::Error),
}

/*
 * Parses and preprepocess all files passed by User
 */
fn user_data_parsing(cli: &Cli) -> RnxContext {
    let mut ctx = RnxContext::default();

    let max_depth = match cli.matches.get_one::<u8>("depth") {
        Some(value) => *value as usize,
        None => 5usize,
    };

    /*
     * Load directories recursively (`-d`)
     */
    for dir in cli.input_directories() {
        let walkdir = WalkDir::new(dir).max_depth(max_depth);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let path = entry.path();
                if let Ok(rinex) = Rinex::from_path(path) {
                    let loading = ctx.load_rinex(path, rinex);
                    if loading.is_err() {
                        warn!(
                            "failed to load RINEX file \"{}\": {}",
                            path.display(),
                            loading.err().unwrap()
                        );
                    }
                } else if let Ok(sp3) = SP3::from_path(path) {
                    let loading = ctx.load_sp3(path, sp3);
                    if loading.is_err() {
                        warn!(
                            "failed to load SP3 file \"{}\": {}",
                            path.display(),
                            loading.err().unwrap()
                        );
                    }
                } else {
                    warn!("non supported file format \"{}\"", path.display());
                }
            }
        }
    }

    /*
     * Load each individual file (`-f`)
     */
    for fp in cli.input_files() {
        let path = Path::new(fp);
        if let Ok(rinex) = Rinex::from_path(path) {
            let loading = ctx.load_rinex(path, rinex);
            if loading.is_err() {
                warn!(
                    "failed to load RINEX file \"{}\": {}",
                    path.display(),
                    loading.err().unwrap()
                );
            }
        } else if let Ok(sp3) = SP3::from_path(path) {
            let loading = ctx.load_sp3(path, sp3);
            if loading.is_err() {
                warn!(
                    "failed to load SP3 file \"{}\": {}",
                    path.display(),
                    loading.err().unwrap()
                );
            }
        } else {
            warn!("non supported file format \"{}\"", path.display());
        }
    }

    /*
     * Preprocess whole context
     */
    preprocess(&mut ctx, &cli);
    ctx
}

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    /*
     * Build context defined by user
     *   Parse all data, determine other useful information
     */
    let cli = Cli::new();

    // User Data parsing
    let mut data_ctx = user_data_parsing(&cli);
    let ctx_position = data_ctx.ground_position();
    let ctx_stem = Context::context_stem(&mut data_ctx);

    // Form context
    let mut ctx = Context {
        name: ctx_stem.clone(),
        data: data_ctx,
        quiet: cli.matches.get_flag("quiet"),
        workspace: {
            /*
             * Supports both an environment variable and
             * a command line opts. Otherwise we use ./workspace directly
             * but its creation must pass.
             * This is documented in Wiki pages.
             */
            let path = match std::env::var("RINEX_WORKSPACE") {
                Ok(path) => Path::new(&path).join(&ctx_stem).to_path_buf(),
                _ => match cli.matches.get_one::<PathBuf>("workspace") {
                    Some(base_dir) => Path::new(base_dir).join(&ctx_stem).to_path_buf(),
                    None => Path::new("WORKSPACE").join(&ctx_stem).to_path_buf(),
                },
            };
            // make sure the workspace is viable and exists, otherwise panic
            create_dir_all(&path).unwrap_or_else(|e| {
                panic!(
                    "failed to create session workspace \"{}\": {:?}",
                    path.display(),
                    e
                )
            });
            info!("session workspace is \"{}\"", path.to_string_lossy());
            path
        },
        rx_ecef: {
            /*
             * Determine and store RX (ECEF) position
             * Either manually defined by User
             *   this is useful in case not a single file has such information
             *   or we want to use a custom location
             * Or with smart determination from all previously parsed data
             *   this is useful in case we don't want to bother
             *   but we must be sure that the OBSRINEX describes the correct location
             */
            match cli.manual_position() {
                Some((x, y, z)) => {
                    let (mut lat, mut lon, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                    lat = rad2deg(lat);
                    lon = rad2deg(lon);
                    info!(
                        "using manually defined position: {:?} [ECEF] (lat={:.5}°, lon={:.5}°",
                        (x, y, z),
                        lat,
                        lon
                    );
                    Some((x, y, z))
                },
                None => {
                    if let Some(data_pos) = ctx_position {
                        let (x, y, z) = data_pos.to_ecef_wgs84();
                        let (mut lat, mut lon, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                        lat = rad2deg(lat);
                        lon = rad2deg(lon);
                        info!(
                            "position as defined in dataset: {:?} [ECEF] (lat={:.5}°, lon={:.5}°",
                            (x, y, z),
                            lat,
                            lon
                        );
                        Some((x, y, z))
                    } else {
                        /*
                         * Dataset does not contain any position,
                         * and User did not specify any.
                         * This is not problematic unless user is interested in
                         * advanced operations, which will most likely fail soon or later.
                         */
                        warn!("no RX position defined");
                        None
                    }
                },
            }
        },
    };

    /*
     * Exclusive opmodes
     */
    match cli.matches.subcommand() {
        Some(("filegen", submatches)) => {
            fops::filegen(&ctx, submatches)?;
        },
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
        Some(("quality-check", submatches)) => {
            qc::qc_report(&ctx, submatches)?;
        },
        Some(("positioning", submatches)) => {
            positioning::precise_positioning(&ctx, submatches)?;
        },
        Some(("tbin", submatches)) => {
            fops::time_binning(&ctx, submatches)?;
        },
        Some(("sub", submatches)) => {
            fops::substract(&ctx, submatches)?;
        },
        _ => error!("no opmode specified!"),
    }
    Ok(())
} // main
