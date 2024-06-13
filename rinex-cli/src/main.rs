//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

//mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod graph;
mod positioning;
mod qc; // QC report generator // plotting operations // file operation helpers // graphical analysis // positioning + CGGTTS opmode

mod preprocessing;
use preprocessing::preprocess;

use rinex_qc::prelude::{Preprocessing, QcContext};

use std::path::Path;
use walkdir::WalkDir;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use rinex::prelude::Rinex;
use sp3::prelude::SP3;

use cli::{Cli, Context, Workspace};

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
fn user_data_parsing(cli: &Cli) -> QcContext {
    let mut ctx = QcContext::default();

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
                    if loading.is_ok() {
                        info!("Loading RINEX file \"{}\"", path.display());
                    } else {
                        warn!(
                            "failed to load RINEX file \"{}\": {}",
                            path.display(),
                            loading.err().unwrap()
                        );
                    }
                } else if let Ok(sp3) = SP3::from_path(path) {
                    let loading = ctx.load_sp3(path, sp3);
                    if loading.is_ok() {
                        info!("Loading SP3 file \"{}\"", path.display());
                    } else {
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
    preprocess(&mut ctx, cli);
    debug!("{:?}", ctx);
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
    let ctx = Context {
        name: ctx_stem.clone(),
        data: data_ctx,
        quiet: cli.matches.get_flag("quiet"),
        workspace: Workspace::new(&ctx_stem, &cli),
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
                        "Manually defined position: {:?} [ECEF] (lat={:.5}°, lon={:.5}°)",
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
                            "Position defined in dataset: {:?} [ECEF] (lat={:.5}°, lon={:.5}°)",
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
                        warn!("No RX position defined");
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
        Some(("merge", submatches)) => {
            fops::merge(&ctx, submatches)?;
        },
        Some(("split", submatches)) => {
            fops::split(&ctx, submatches)?;
        },
        Some(("qc", submatches)) => {
            qc::qc_report(&ctx, submatches)?;
        },
        Some(("ppp", submatches)) => {
            positioning::precise_positioning(&ctx, submatches)?;
        },
        Some(("tbin", submatches)) => {
            fops::time_binning(&ctx, submatches)?;
        },
        Some(("diff", submatches)) => {
            fops::diff(&ctx, submatches)?;
        },
        Some(("positioning", submatches)) => {
            panic!("not supported");
        },
        _ => panic!("no opmode specified!"),
    }

    if !ctx.quiet {
        ctx.workspace.open_with_web_browser();
    }
    Ok(())
} // main
