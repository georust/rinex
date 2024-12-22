//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Homepage: <https://github.com/georust/rinex-cli>

extern crate gnss_rs as gnss;

use std::io::Write;
use std::{io::Error as IoError, path::Path};

//mod analysis;
mod cli;
mod fops;
mod preprocessing;

use preprocessing::preprocess;

// mod report;
// use report::Report;

use rinex::prelude::{FormattingError as RinexFormattingError, ParsingError as RinexParsingError};
use rinex_qc::prelude::{MergeError, QcContext, QcError, QcExtraPage, Render};

use crate::fops::{
    diff as fops_diff, filegen as fops_filegen, merge as fops_merge, split as fops_split,
    tbin as fops_tbin,
};

use walkdir::WalkDir;

use cli::{Cli, CliContext};

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
    #[error("qc error")]
    QcError(#[from] QcError),
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
    let cfg = cli.qc_config();

    let mut ctx =
        QcContext::new(cfg).unwrap_or_else(|e| panic!("failed to initialize new context {}", e));

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

    // Possible requested data transformation(s)
    if cli.rnx2crx() {
        info!("rnx2crx: internal compression");
        ctx.rnx2crx_mut();
    }

    if cli.crx2rnx() {
        info!("crx2rnx: internal decompression");
        ctx.crx2rnx_mut();
    }

    // Special print in case of RTK
    // it helps differentiate between remote and local context
    match cli.matches.subcommand() {
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
    let qc_ctx = user_data_parsing(
        &cli,
        cli.rover_files(),
        cli.rover_directories(),
        max_recursive_depth,
        true,
    );

    let mut ctx = CliContext {
        quiet: cli.quiet(),
        qc_context: qc_ctx,
    };

    // When output product needs to be generated (other than simple report.html)
    // we put them into seperate folders
    if cli.has_fops_output_product() {
        for (k, _) in ctx.qc_context.obs_dataset.iter() {
            ctx.qc_context.create_subdir(&k.name)?;
        }
        for (k, _) in ctx.qc_context.meteo_dataset.iter() {
            ctx.qc_context.create_subdir(&k.name)?;
        }
    }

    let mut extra_pages = Vec::<QcExtraPage>::new();

    // Exclusive opmodes (mostlye file operations)
    // We handle them right here and exit early.
    // Avoids proceed to report synthesis (default opmode).
    match cli.matches.subcommand() {
        Some(("filegen", submatches)) => {
            fops_filegen(&ctx.qc_context, &cli.matches, submatches)?;
            return Ok(());
        },
        Some(("merge", submatches)) => {
            fops_merge(&ctx.qc_context, &cli, submatches)?;
            return Ok(());
        },
        Some(("split", submatches)) => {
            fops_split(&ctx.qc_context, &cli, submatches)?;
            return Ok(());
        },
        Some(("tbin", submatches)) => {
            fops_tbin(&ctx.qc_context, &cli, submatches)?;
            return Ok(());
        },
        Some(("diff", submatches)) => {
            fops_diff(&mut ctx.qc_context, &cli, submatches)?;
            return Ok(());
        },
        // Some(("ppp", submatches)) => {
        //     let chapter = positioning::precise_positioning(&cli, &ctx, false, submatches)?;
        //     extra_pages.push(chapter);
        // },
        // Some(("rtk", submatches)) => {
        //     let chapter = positioning::precise_positioning(&cli, &ctx, true, submatches)?;
        //     extra_pages.push(chapter);
        // },
        _ => {},
    }

    // Report synthesis opmode (=Default opmode)

    let report = ctx.qc_context.report_synthesis();
    let html = report.render().into_string();

    let mut fd = ctx
        .qc_context
        .create_file("index.html")
        .unwrap_or_else(|e| panic!("failed to create file in workspace: {}", e));

    write!(fd, "{}", html)?;

    if !cli.quiet() {
        ctx.qc_context.open_workspace_with_browser();
    }

    Ok(())
} // main
