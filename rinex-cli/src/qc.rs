//! File Quality opmode
use clap::ArgMatches;
use log::info;
use std::fs::{read_to_string, File};
use std::io::Write;

use crate::cli::Context;
use crate::Error;
use rinex_qc::prelude::{QcConfig, QcExtraPage, QcReport, Render};
use std::collections::HashMap;

pub fn qc_report(ctx: &Context, extra_pages: Vec<QcExtraPage>) -> Result<(), Error> {
    let cfg = QcConfig::default();
    info!("using default QC configuration: {:#?}", cfg);

    let html = QcReport::new(&ctx.data, cfg).render().into_string();
    let report_path = ctx.workspace.root.join("QC.html");

    let mut fd = File::create(&report_path).map_err(|_| Error::QcReportCreationError)?;

    write!(fd, "{}", html).expect("failed to render HTML report");
    info!("QC report \"{}\" has been generated", report_path.display());
    Ok(())
}
