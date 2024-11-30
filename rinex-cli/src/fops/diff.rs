use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::{Rinex, RinexType};
use rinex_qc::prelude::ProductType;
use std::path::PathBuf;

/// Observation RINEX (a -b) special differential operation.
/// Dumps result into workspace.
pub fn diff(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;

    for file in ctx_data.files_iter(Some(ProductType::Observation), None) {
        let path_b = matches.get_one::<PathBuf>("file").unwrap();

        let extension = path_b
            .extension()
            .expect("failed to determine output file extension");

        let rinex_b = if extension.eq("gz") {
            Rinex::from_gzip_file(&path_b).expect("failed to load diff(a-b*) file")
        } else {
            Rinex::from_file(&path_b).expect("failed to load diff(a-b*) file")
        };

        let rinex_a = ctx_data
            .get_rinex_data_mut(RinexType::ObservationData)
            .expect("RINEX (A-B) requires Observation RINEX");

        rinex_a.substract_mut(&rinex_b);
    }

    let mut extension = String::new();

    let filename = path_a
        .file_stem()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    if filename.contains('.') {
        /* .crx.gz case */
        let mut iter = filename.split('.');
        let _filename = iter
            .next()
            .expect("failed to determine output file name")
            .to_string();
        extension.push_str(iter.next().expect("failed to determine output file name"));
        extension.push('.');
    }

    let file_ext = path_a
        .extension()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    extension.push_str(&file_ext);

    let fullpath = ctx
        .workspace
        .root
        .join(format!("DIFFERENCED.{}", extension))
        .to_string_lossy()
        .to_string();

    rinex_c.to_file(&fullpath)?;

    info!("OBS RINEX \"{}\" has been generated", fullpath);
    Ok(())
}
