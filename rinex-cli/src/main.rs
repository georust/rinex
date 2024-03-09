//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

mod analysis; // basic analysis
mod cli; // command line interface
mod fops;
mod graph;
mod identification; // high level identification/macros
mod positioning;
mod qc; // QC report generator // plotting operations // file operation helpers // graphical analysis // positioning + CGGTTS opmode

mod preprocessing;
use preprocessing::preprocess;

extern crate gnss_rs as gnss;
extern crate gnss_rtk as rtk;

use cli::{Cli, Context};

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
 * Parse all input data
 */
fn data_parsing(cli: &Cli) -> RnxContext {
    let mut ctx = RnxContext::default();

    let recursive_depth = match cli.matches.get_one::<u8>("depth") {
        Some(depth) => *depth as usize,
        None => 5usize,
    };

    /* load all directories recursively, one by one */
    for dir in cli.input_directories() {
        let walkdir = WalkDir::new(dir).max_depth(max_depth);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let path = entry.path();
                if let Ok(rnx) = Rinex::from_path(path) {
                    if ctx.load_rinex(path, rnx).is_err() {
                        warn!(
                            "failed to load \"{}\": {}",
                            path.display(),
                            ret.err().unwrap()
                        );
                    }
                } else {
                    error!("malformed RINEX \"{}\"", path.display());
                }
            }
        }
    }
    // load individual files, if any
    for filepath in cli.input_files() {
        let ret = data.load(&Path::new(filepath).to_path_buf());
        if ret.is_err() {
            warn!("failed to load \"{}\": {}", filepath, ret.err().unwrap());
        }
    }
}

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Build context defined by user
    let cli = Cli::new();

    // Form context
    let mut data = data_parsing(&cli);
    let ctx = Context::from_cli(&cli).with_data_context(data);

    /*
     * Preprocessing
     */
    preprocess(&mut ctx.data, &cli);

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
        Some(("sub", submatches)) => {
            fops::substract(&ctx, submatches)?;
        },
        Some(("tbin", submatches)) => {
            fops::time_binning(&ctx, submatches)?;
        },
        _ => error!("no opmode specified!"),
    }
    Ok(())
} // main
