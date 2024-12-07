//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

//mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod positioning;

mod preprocessing;
use preprocessing::preprocess;

mod report;
use report::Report;

use sp3::prelude::SP3;

use rinex::prelude::{
    FormattingError as RinexFormattingError, ParsingError as RinexParsingError, Rinex,
};

use rinex_qc::prelude::{MergeError, QcContext, QcExtraPage};

use std::{io::Error as IoError, path::Path};

use walkdir::WalkDir;

extern crate gnss_rs as gnss;

use cli::{Cli, Context, RemoteReferenceSite, Workspace};

use map_3d::{ecef2geodetic, Ellipsoid};

#[cfg(feature = "csv")]
use csv::Error as CsvError;

use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error")]
    StdioError(#[from] IoError),
    #[error("rinex parsing")]
    RinexParsing(#[from] RinexParsingError),
    #[error("rinex formatting")]
    RinexFormatting(#[from] RinexFormattingError),
    #[error("missing OBS RINEX")]
    MissingObservationRinex,
    #[error("missing (BRDC) NAV RINEX")]
    MissingNavigationRinex,
    #[error("missing IONEX")]
    MissingIONEX,
    #[error("missing Meteo RINEX")]
    MissingMeteoRinex,
    #[error("missing Clock RINEX")]
    MissingClockRinex,
    #[error("positioning solver error")]
    PositioningSolverError(#[from] positioning::Error),
    #[cfg(feature = "csv")]
    #[error("csv export error")]
    CsvError(#[from] CsvError),
    #[error("merge error")]
    Merge(#[from] MergeError),
}

/// Parses and preprepocess all files passed by User
fn user_data_parsing(
    cli: &Cli,
    single_files: Vec<&String>,
    directories: Vec<&String>,
    max_depth: usize,
    is_rover: bool,
) -> QcContext {
    let mut ctx =
        QcContext::new().unwrap_or_else(|e| panic!("failed to initialize new context {}", e));

    // recursive dir loader
    for dir in directories.iter() {
        let walkdir = WalkDir::new(dir).max_depth(max_depth);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let path = entry.path();

                let file_ext = path
                    .extension()
                    .expect("failed to determine file extension")
                    .to_string_lossy()
                    .to_string();

                let gzip_encoded = file_ext == "gz";

                // we support gzip encoding
                if gzip_encoded {
                    match ctx.load_gzip_file(path) {
                        Ok(()) => {},
                        Err(e) => {
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();
                            error!("\"{}\" File loading error: {}", filename, e);
                        },
                    }
                } else {
                    match ctx.load_file(path) {
                        Ok(()) => {},
                        Err(e) => {
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();
                            error!("\"{}\" File loading error: {}", filename, e);
                        },
                    }
                }
            }
        }
    }

    // load individual files
    for fp in single_files.iter() {
        let path = Path::new(fp);

        let file_ext = path
            .extension()
            .expect("failed to determine file extension")
            .to_string_lossy()
            .to_string();

        let gzip_encoded = file_ext == "gz";

        // we support gzip encoding
        if gzip_encoded {
            match ctx.load_gzip_file(path) {
                Ok(()) => {},
                Err(e) => {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    error!("\"{}\" File loading error: {}", filename, e);
                },
            }
        } else {
            match ctx.load_file(path) {
                Ok(()) => {},
                Err(e) => {
                    let filename = path.file_name().unwrap_or_default().to_string_lossy();
                    error!("\"{}\" File loading error: {}", filename, e);
                },
            }
        }
    }

    // Preprocessing: Resampling, Filtering..
    preprocess(&mut ctx, cli);

    match cli.matches.subcommand() {
        // Special print in case of RTK
        // it helps differentiate between remote and local context
        Some(("rtk", _)) => {
            if is_rover {
                debug!("ROVER Dataset: {:?}", ctx);
            } else {
                error!("BASE STATION Dataset: {:?}", ctx);
            }
        },
        _ => {
            // Default print (normal use case)
            // local context (=rover) only
            debug!("{:?}", ctx);
        },
    }

    ctx
}

pub fn main() -> Result<(), Error> {
    // logs builder
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Build data context: parse all data
    // and determine useful information for this session
    let cli = Cli::new();
    let max_recursive_depth = cli.recursive_depth();

    // User (ROVER) Data parsing
    let mut data_ctx = user_data_parsing(
        &cli,
        cli.rover_files(),
        cli.rover_directories(),
        max_recursive_depth,
        true,
    );

    let ctx_position = data_ctx.reference_position();
    let ctx_stem = Context::context_stem(&mut data_ctx);

    /*
     * Determine and store RX (ECEF) position
     * Either manually defined by User
     *   this is useful in case not a single file has such information
     *   or we want to use a custom location
     * Or with smart determination from all previously parsed data
     *   this is useful in case we don't want to bother
     *   but we must be sure that the OBSRINEX describes the correct location
     */
    let rx_ecef = match cli.manual_position() {
        Some((x, y, z)) => {
            let (lat, lon, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
            let (lat_ddeg, lon_ddeg) = (lat.to_degrees(), lon.to_degrees());
            info!(
                "Manually defined position: {:?} [ECEF] (lat={:.5}°, lon={:.5}°)",
                (x, y, z),
                lat_ddeg,
                lon_ddeg
            );
            Some((x, y, z))
        },
        None => {
            if let Some(data_pos) = ctx_position {
                let (x, y, z) = data_pos.to_ecef_wgs84();
                let (lat, lon, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                let (lat_ddeg, lon_ddeg) = (lat.to_degrees(), lon.to_degrees());
                info!(
                    "Position defined in dataset: {:?} [ECEF] (lat={:.5}°, lon={:.5}°)",
                    (x, y, z),
                    lat_ddeg,
                    lon_ddeg
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
    };

    // Form context
    let mut ctx = Context {
        name: ctx_stem.clone(),
        data: data_ctx,
        reference_site: {
            match cli.matches.subcommand() {
                // Remote reference site (Base Station) User specs.
                Some(("rtk", _)) => {
                    let data = user_data_parsing(
                        &cli,
                        cli.base_station_files(),
                        cli.base_station_directories(),
                        max_recursive_depth,
                        false,
                    );
                    // We currently require remote site
                    // to have its geodetic marker declared
                    if let Some(reference_point) = data.reference_position() {
                        let (base_x0_m, base_y0_m, base_z0_m) = reference_point.to_ecef_wgs84();
                        if let Some(rx_ecef) = rx_ecef {
                            let baseline_m = ((base_x0_m - rx_ecef.0).powi(2)
                                + (base_y0_m - rx_ecef.1).powi(2)
                                + (base_z0_m - rx_ecef.2).powi(2))
                            .sqrt();
                            if baseline_m > 1000.0 {
                                info!(
                                    "Rover / Reference site baseline projection: {:.3}km",
                                    baseline_m / 1000.0
                                );
                            } else {
                                info!(
                                    "Rover / Reference site baseline projection: {:.3}m",
                                    baseline_m
                                );
                            }
                        }
                        Some(RemoteReferenceSite {
                            data,
                            rx_ecef: Some((base_x0_m, base_y0_m, base_z0_m)),
                        })
                    } else {
                        error!("remote site does not have its geodetic marker defined: current CLI limitation.");
                        None
                    }
                },
                _ => None,
            }
        },
        quiet: cli.matches.get_flag("quiet"),
        workspace: Workspace::new(&ctx_stem, &cli),
        rx_ecef,
    };

    // On File Operations (Data synthesis)
    //  prepare one subfolder to store the output products
    if cli.has_fops_output_product() {
        ctx.create_output_products_dir();
    }

    /*
     * Exclusive opmodes
     */
    let mut extra_pages = Vec::<QcExtraPage>::new();

    match cli.matches.subcommand() {
        /*
         *  File operations abort here and do not windup in analysis opmode.
         *  Users needs to then deploy analysis mode on previously generated files.
         */
        Some(("filegen", submatches)) => {
            fops::filegen(&ctx, &cli.matches, submatches)?;
            return Ok(());
        },
        Some(("merge", submatches)) => {
            fops::merge(&ctx, submatches)?;
            return Ok(());
        },
        Some(("split", submatches)) => {
            fops::split(&ctx, submatches)?;
            return Ok(());
        },
        Some(("tbin", submatches)) => {
            fops::time_binning(&ctx, &cli.matches, submatches)?;
            return Ok(());
        },
        Some(("diff", submatches)) => {
            fops::diff(&mut ctx, submatches)?;
            return Ok(());
        },
        Some(("ppp", submatches)) => {
            let chapter = positioning::precise_positioning(&cli, &ctx, false, submatches)?;
            extra_pages.push(chapter);
        },
        Some(("rtk", submatches)) => {
            let chapter = positioning::precise_positioning(&cli, &ctx, true, submatches)?;
            extra_pages.push(chapter);
        },
        _ => {},
    }

    // report
    let cfg = cli.qc_config();
    let mut report = Report::new(&cli, &ctx, cfg);

    // customization
    for extra in extra_pages {
        report.customize(extra);
    }

    // generation
    report.generate(&cli, &ctx)?;

    if !ctx.quiet {
        ctx.workspace.open_with_web_browser();
    }

    Ok(())
} // main
