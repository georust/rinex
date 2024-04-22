//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

//mod analysis; // basic analysis
mod cli; // command line interface
         // mod fops; // file operations
         // mod graph; // plotting
         // mod identification; // high level identification/macros
         // mod positioning; // positioning + CGGTTS opmode
         // mod qc; // QC report generator

mod preprocessing;
// use preprocessing::preprocess;

use rinex_qc::prelude::DataContext;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use cli::{Cli, Session};
use env_logger::{Builder, Target};
use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};
use rinex::prelude::Rinex;
use sp3::prelude::SP3;

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
    // #[error("positioning solver error")]
    // PositioningSolverError(#[from] positioning::Error),
}

/*
 * Parses and preprepocess all files passed by User
 */
fn user_data_parsing(cli: &Cli) -> DataContext {
    let mut ctx = DataContext::default();

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
                //let path = entry.path();
                //if let Ok(rinex) = Rinex::from_path(path) {
                //    let loading = ctx.load_rinex(path, rinex);
                //    if loading.is_ok() {
                //        info!("Loading RINEX file \"{}\"", path.display());
                //    } else {
                //        warn!(
                //            "failed to load RINEX file \"{}\": {}",
                //            path.display(),
                //            loading.err().unwrap()
                //        );
                //    }
                //} else if let Ok(sp3) = SP3::from_path(path) {
                //    let loading = ctx.load_sp3(path, sp3);
                //    if loading.is_ok() {
                //        info!("Loading SP3 file \"{}\"", path.display());
                //    } else {
                //        warn!(
                //            "failed to load SP3 file \"{}\": {}",
                //            path.display(),
                //            loading.err().unwrap()
                //        );
                //    }
                //} else {
                //warn!("non supported file format \"{}\"", path.display());
                //}
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
            warn!("Non supported file \"{}\"", path.display());
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

    /*
     * Build context defined by user
     *   Parse all data, determine other useful information
     */
    let cli = Cli::new();

    // User Data parsing
    let data_ctx = user_data_parsing(&cli);

    let first_epoch = data_ctx.first_epoch().unwrap_or_else(|| {
        panic!(
            "Failed to generate a session - fileset is most likely empty.
You need to load at least one file to analyze.
ANTEX is a special case that cannot be loaded by itself, it can only enhance a fileset."
        );
    });

    // create a session
    let session = Session {
        quiet: cli.matches.get_flag("quiet"),
        workspace: {
            /*
             * Supports both an environment variable and
             * a command line opts. Otherwise we use ./workspace directly
             * but its creation must pass.
             * This is documented in Wiki pages.
             */
            let path = match std::env::var("RINEX_WORKSPACE") {
                Ok(path) => Path::new(&path).to_path_buf(),
                _ => match cli.matches.get_one::<PathBuf>("workspace") {
                    Some(base_dir) => Path::new(base_dir).to_path_buf(),
                    None => Path::new("WORKSPACE").to_path_buf(),
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
        geodetic_marker: {
            match cli.manual_geodetic_marker() {
                Some(position) => {
                    let (lat, long, _) = position.to_geodetic();
                    let (x, y, z) = position.to_ecef_wgs84();
                    info!(
                        "Manually defined geodetic marker: {:?} [ECEF] (lat={:.5}°, lon={:.5}°",
                        (x, y, z),
                        rad2deg(lat),
                        rad2deg(long)
                    );
                    Some(position)
                },
                None => {
                    if let Some(marker) = data_ctx.geodetic_marker_position() {
                        let (lat, long, _) = marker.to_geodetic();
                        let (x, y, z) = marker.to_ecef_wgs84();
                        info!(
                            "Geodetic marker defined dataset: {:?} [ECEF] (lat={:.5}°, lon={:.5}°",
                            (x, y, z),
                            rad2deg(lat),
                            rad2deg(long)
                        );
                        Some(marker)
                    } else {
                        /*
                         * Dataset does not contain any position,
                         * and User did not specify any.
                         * This is not problematic unless requested opmode needs it.
                         */
                        warn!("No geodetic marker identified");
                        None
                    }
                },
            }
        },
        data: data_ctx,
    };

    for year in session.data.yearly_iter() {
        println!("YEAR: {:04}", year);
    }
    for (year, doy) in session.data.yearly_doy_iter() {
        println!("YEAR: {:04} | DOY {:03}", year, doy);
    }

    /*
     * Exclusive opmodes
     */
    match cli.matches.subcommand() {
        Some(("filegen", submatches)) => {
            // fops::filegen(&ctx, submatches)?;
        },
        Some(("graph", submatches)) => {
            // graph::graph_opmode(&ctx, submatches)?;
        },
        Some(("identify", submatches)) => {
            // identification::dataset_identification(&ctx.data, submatches);
        },
        Some(("merge", submatches)) => {
            // fops::merge(&ctx, submatches)?;
        },
        Some(("split", submatches)) => {
            // fops::split(&ctx, submatches)?;
        },
        Some(("positioning", submatches)) => {
            // positioning::precise_positioning(&ctx, submatches)?;
        },
        Some(("tbin", submatches)) => {
            // fops::time_binning(&ctx, submatches)?;
        },
        Some(("sub", submatches)) => {
            // fops::substract(&ctx, submatches)?;
        },
        _ => {
            info!("running basic QC");
            // qc::qc_report(&ctx, submatches)?;
        },
    }

    session.finalize();
    Ok(())
} // main
