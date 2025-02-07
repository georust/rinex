//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod cli; // command line interface
mod fops; // file operations
mod positioning; // post processed positioning
mod preprocessing; // preprocessing
mod report; // custom reports

use preprocessing::preprocess;
use report::Report;

use rinex::prelude::{FormattingError as RinexFormattingError, ParsingError as RinexParsingError};
use rinex_qc::prelude::{QcContext, QcExtraPage};

use std::path::Path;
use walkdir::WalkDir;

extern crate gnss_rs as gnss;

use rinex::prelude::{nav::Orbit, qc::MergeError, Rinex};

use sp3::prelude::SP3;

use cli::{Cli, Context, RemoteReferenceSite, Workspace};

#[cfg(feature = "csv")]
use csv::Error as CsvError;

use env_logger::{Builder, Target};

#[macro_use]
extern crate log;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error")]
    StdioError(#[from] std::io::Error),
    #[error("missing OBS RINEX")]
    MissingObservationRinex,
    #[error("RINEX parsing error: {0}")]
    RinexParsing(#[from] RinexParsingError),
    #[error("RINEX formatting error: {0}")]
    RinexFormatting(#[from] RinexFormattingError),
    #[error("Qc merge error: {0}")]
    Merge(#[from] MergeError),
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
}

/// Parses and preprepocess all files passed by User
fn user_data_parsing(
    cli: &Cli,
    single_files: Vec<&String>,
    directories: Vec<&String>,
    max_depth: usize,
    is_rover: bool,
) -> QcContext {
    let mut ctx = QcContext::new(cli.jpl_bpc_update())
        .unwrap_or_else(|e| panic!("failed to initialize a context: {}", e));

    // recursive dir loader
    for dir in directories.iter() {
        let walkdir = WalkDir::new(dir).max_depth(max_depth);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_dir() {
                let extension = path
                    .extension()
                    .unwrap_or_else(|| {
                        panic!("failed to determine file extension: \"{}\"", path.display())
                    })
                    .to_string_lossy()
                    .to_string();

                if extension == "gz" {
                    if let Ok(rinex) = Rinex::from_gzip_file(path) {
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
                } else {
                    if let Ok(rinex) = Rinex::from_file(path) {
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
                    }
                }
            }
        }
    }

    // load individual files
    for fp in single_files.iter() {
        let path = Path::new(fp);

        let extension = path
            .extension()
            .unwrap_or_else(|| panic!("failed to determine file extension: \"{}\"", path.display()))
            .to_string_lossy()
            .to_string();

        if extension == "gz" {
            if let Ok(rinex) = Rinex::from_gzip_file(path) {
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
        } else {
            if let Ok(rinex) = Rinex::from_file(path) {
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

    /// Preprocessing
    preprocess(&mut ctx, cli);

    match cli.matches.subcommand() {
        Some(("rtk", _)) => {
            if is_rover {
                debug!("ROVER Dataset: {:?}", ctx);
            } else {
                error!("BASE STATION Dataset: {:?}", ctx);
            }
        },
        _ => {
            debug!("{:?}", ctx);
        },
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
    let max_recursive_depth = cli.recursive_depth();

    // User (ROVER) Data parsing
    let mut data_ctx = user_data_parsing(
        &cli,
        cli.rover_files(),
        cli.rover_directories(),
        max_recursive_depth,
        true,
    );

    let ctx_orbit = data_ctx.reference_rx_orbit();
    let ctx_stem = Context::context_stem(&mut data_ctx);

    // Input context
    let ctx = Context {
        name: ctx_stem.clone(),
        rx_orbit: {
            // possible reference point
            if let Some(rx_orbit) = data_ctx.reference_rx_orbit() {
                let posvel = rx_orbit.to_cartesian_pos_vel();
                let (x0_km, y0_km, z0_km) = (posvel[0], posvel[1], posvel[2]);
                let (lat_ddeg, long_ddeg, _) = rx_orbit
                    .latlongalt()
                    .unwrap_or_else(|e| panic!("latlongalt - physical error: {}", e));
                info!("reference point identified: {:.5E}km, {:.5E}km, {:.5E}km (lat={:.5}°, long={:.5}°)", x0_km, y0_km, z0_km, lat_ddeg, long_ddeg);
            } else {
                warn!("no reference point identifed");
            }
        },
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
                    Some(RemoteReferenceSite {
                        data,
                        rx_ecef: Some((0.0, 0.0, 0.0)),
                    })
                },
                _ => None,
            }
        },
        quiet: cli.matches.get_flag("quiet"),
        workspace: Workspace::new(&ctx_stem, &cli),
    };

    // ground reference point
    match ctx.rx_orbit {
        Some(orbit) => {
            if let Some(obs_rinex) = ctx.data.observation() {
                if let Some(t0) = obs_rinex.first_epoch() {
                    if let Some(rx_orbit) = cli.manual_rx_orbit(t0, ctx.data.earth_cef) {
                        let posvel = rx_orbit.to_cartesian_pos_vel();
                        let (x0_km, y0_km, z0_km) = (posvel[0], posvel[1], posvel[2]);
                        let (lat_ddeg, long_ddeg, _) = rx_orbit
                            .latlongalt()
                            .unwrap_or_else(|e| panic!("latlongalt - physical error: {}", e));
                        info!("reference point manually overwritten: {:.5E}km, {:.5E}km, {:.5E}km (lat={:.5}°, long={:.5}°)", x0_km, y0_km, z0_km, lat_ddeg, long_ddeg);
                        ctx.rx_orbit = Some(rx_orbit);
                    }
                }
            }
        },
        None => {
            if let Some(obs_rinex) = ctx.data.observation() {
                if let Some(t0) = obs_rinex.first_epoch() {
                    if let Some(rx_orbit) = cli.manual_rx_orbit(t0, ctx.data.earth_cef) {
                        let posvel = rx_orbit.to_cartesian_pos_vel();
                        let (x0_km, y0_km, z0_km) = (posvel[0], posvel[1], posvel[2]);
                        let (lat_ddeg, long_ddeg, _) = rx_orbit
                            .latlongalt()
                            .unwrap_or_else(|e| panic!("latlongalt - physical error: {}", e));
                        info!("manually defined reference point: {:.5E}km, {:.5E}km, {:.5E}km (lat={:.5}°, long={:.5}°)", x0_km, y0_km, z0_km, lat_ddeg, long_ddeg);
                        ctx.rx_orbit = Some(rx_orbit);
                    }
                }
            } else {
                error!("manual definition of a reference point requires OBS RINEX");
            }
        },
    }

    // Prepare for output productes (on any FOPS)
    if cli.has_fops_output_product() {
        ctx.workspace.create_subdir("OUTPUT");
    }

    // Exclusive opmodes to follow
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
            fops::diff(&ctx, submatches)?;
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

    // synthesis
    report.generate(&cli, &ctx)?;

    if !ctx.quiet {
        ctx.workspace.open_with_web_browser();
    }

    Ok(())
} // main
