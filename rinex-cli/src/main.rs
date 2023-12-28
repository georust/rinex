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

pub fn main() -> Result<(), Error> {
    let mut builder = Builder::from_default_env();
    builder
        .target(Target::Stdout)
        .format_timestamp_secs()
        .format_module_path(false)
        .init();

    // Build context defined by user
    let cli = Cli::new();
    let mut ctx = Context::from_cli(&cli)?;

    /*
     * Preprocessing
     */
    preprocess(&mut ctx.data, &cli);

    /*
     * Exclusive opmodes
     */
    match cli.matches.subcommand() {
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
