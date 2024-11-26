use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::{Rinex, RinexType};
use rinex_qc::prelude::ProductType;
use std::path::PathBuf;

/// Runs the RINEX[A]-RINEX[B] differential operation
/// and dumps output result into the workspace.
pub fn diff(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let path_a = ctx_data
        .files(ProductType::Observation)
        .expect("failed to determine output file name")
        .first()
        .unwrap();

    let path_b = matches.get_one::<PathBuf>("file").unwrap();

    let path_b = path_b.to_string_lossy().to_string();
    let rinex_b = Rinex::from_file(&path_b)
        .unwrap_or_else(|_| panic!("failed to load {}: invalid RINEX", path_b));

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx_data
                .observation()
                .expect("RINEX (A) - (B) requires OBS RINEX files");

            //TODO: change this to crnx2rnx_mut()
            rinex_a.crnx2rnx().substract(&rinex_b.crnx2rnx())
        },
        t => panic!("operation not feasible for {}", t),
    };

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
