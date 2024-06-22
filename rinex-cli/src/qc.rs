//! File Quality opmode
use clap::ArgMatches;
use log::info;
use std::fs::{read_to_string, File};
use std::io::Write;

use crate::cli::Context;
use crate::Error;
use rinex_qc::prelude::{QcConfig, QcReport, Render};

pub fn qc_report(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|_| panic!("failed to read QC configuration: permission denied"));
            let cfg = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("failed to parse QC configuration: invalid content"));
            info!("using custom QC configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let cfg = QcConfig::default();
            info!("using default QC configuration: {:#?}", cfg);
            cfg
        },
    };

    /*
     * print more infos
     */
    //info!("Classification method : {:?}", cfg.classification);
    //info!("Reference position    : {:?}", cfg.ground_position);
    //info!("Minimal SNR           : {:?}", cfg.min_snr_db);
    //info!("Elevation mask        : {:?}", cfg.elev_mask);
    //info!("Sampling gap tolerance: {:?}", cfg.gap_tolerance);

    let html = QcReport::new(&ctx.data, cfg).render().into_string();
    let report_path = ctx.workspace.root.join("QC.html");

    let mut fd = File::create(&report_path).map_err(|_| Error::QcReportCreationError)?;

    write!(fd, "{}", html).expect("failed to render HTML report");
    info!("QC report \"{}\" has been generated", report_path.display());
    Ok(())
}
