//! File Quality opmode
use clap::ArgMatches;
use log::info;
use std::fs::{read_to_string, File};
use std::io::Write;

use crate::cli::Context;
use crate::Error;
use rinex_qc::prelude::{QcConfig, QcReport, Render};

use maud::Render;
use std::collections::HashMap;

pub fn qc_report(
    ctx: &Context,
    custom_chapters: HashMap<String, Box<dyn Render>>,
) -> Result<(), Error> {
    let cfg = QcConfig::default();
    info!("using default QC configuration: {:#?}", cfg);

    let html = QcReport::new(&ctx.data, cfg).render().into_string();
    let report_path = ctx.workspace.root.join("QC.html");

    let mut fd = File::create(&report_path).map_err(|_| Error::QcReportCreationError)?;

    write!(fd, "{}", html).expect("failed to render HTML report");
    info!("QC report \"{}\" has been generated", report_path.display());
    Ok(())
}
