use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;

use crate::fops::custom_prod_attributes;
use rinex::prelude::{Rinex, Version};
use rinex_qc::prelude::ProductType;
use std::path::PathBuf;

/// Observation RINEX (a -b) special differential operation.
/// Dumps result into workspace.
pub fn diff(ctx: &mut Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &mut ctx.data;

    // retrieve and parse (B) input
    let path_b = matches.get_one::<PathBuf>("file").unwrap();

    let extension = path_b
        .extension()
        .expect("failed to determine output file extension");

    let gzip_encoded = extension.eq("gz");

    let rinex_b = if gzip_encoded {
        Rinex::from_gzip_file(&path_b).expect("failed to load diff(a-b*) file")
    } else {
        Rinex::from_file(&path_b).expect("failed to load diff(a-b*) file")
    };

    // retrieve and modify (A) OBSERVATION RINEX (only!)
    let rinex_a = ctx_data
        .get_rinex_data_mut(ProductType::Observation)
        .expect("RINEX (A-B) requires Observation RINEX");

    rinex_a.observation_substract_mut(&rinex_b);

    let prod = custom_prod_attributes(&rinex_a, matches);

    // determine new revision
    let major_a = rinex_a.header.version.major;
    let major_b = rinex_b.header.version.major;
    let major = major_a.min(major_b);

    let minor_a = rinex_b.header.version.minor;
    let minor_b = rinex_b.header.version.minor;
    let minor = minor_a.min(minor_b);

    rinex_a.header.version = Version::new(major, minor);

    let short_filename = if major > 2 { false } else { true };

    // grab standardized name
    // (remove potential .crx / .rnx extension in modern case)
    // we handle it ourselves in this custom name
    let standardized = rinex_a.standard_filename(short_filename, None, Some(prod));
    let standardized_len = standardized.len();

    let mut filename = String::new();

    if major > 2 {
        filename.push_str(&standardized[..standardized_len - 3]);
    } else {
        filename.push_str(&standardized);
    }

    // Ouptut is (A-RX(b)_diff + extension)
    if let Some(rcvr) = &rinex_b.header.rcvr {
        filename.push('-');
        filename.push_str(&rcvr.model.to_ascii_uppercase());
        filename.push_str(&rcvr.sn);
    }

    filename.push_str("_diff");

    let is_crinex = if let Some(obs_b) = &rinex_b.header.obs {
        obs_b.crinex.is_some()
    } else {
        false
    };

    if major > 2 {
        if is_crinex {
            filename.push_str(".crx");
        } else {
            filename.push_str(".rnx");
        }
    }

    // format also follows (B) input
    if gzip_encoded {
        filename.push_str(".gz");
    }

    let fullpath = ctx.workspace.root.join(&filename);

    // Dump data and exit
    rinex_a.to_file(&fullpath)?;
    info!(
        "OBS RINEX \"{}\" has been generated",
        fullpath.to_string_lossy()
    );
    Ok(())
}
