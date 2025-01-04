use clap::ArgMatches;

use crate::{cli::Cli, Error};

use rinex::prelude::{Rinex, RinexType};
use std::path::PathBuf;

use rinex_qc::prelude::QcContext;

/// Observation RINEX (a -b) special differential operation.
/// Dumps result into workspace.
pub fn diff(ctx: &mut QcContext, cli: &Cli, matches: &ArgMatches) -> Result<(), Error> {
    let gzip_encoding = cli.gzip_encoding();
    let short_rinex_name = cli.short_rinex_file_name();
    let csv = matches.get_flag("csv");

    // retrieve and parse (B) input
    let path_b = matches.get_one::<PathBuf>("file").unwrap();

    let extension = path_b
        .extension()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    let gzip_encoded = extension.eq("gz");

    let suffix = if gzip_encoding { Some(".gz") } else { None };

    // parse input
    let rinex_b = if gzip_encoded {
        Rinex::from_gzip_file(&path_b)
    } else {
        Rinex::from_file(&path_b)
    };

    let rinex_b = rinex_b.unwrap_or_else(|e| panic!("failed to parse provided file: {}", e));

    // prepare for output
    match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            for (obs_meta, _) in &ctx.obs_dataset {
                ctx.create_subdir(&obs_meta.meta.name)
                    .unwrap_or_else(|e| panic!("failed to generate output dir: {}", e));
            }
        },
        _ => {},
    }

    // diff only applies to
    //  METEO; DORIS and OBS RINex
    match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            for (obs_meta, rinex) in &mut ctx.obs_dataset {
                rinex.observation_substract_mut(&rinex_b);
                let auto_generated_name = rinex.standard_filename(short_rinex_name, suffix, None);

                let path = ctx.cfg.workspace.join(&obs_meta.meta.name).join(auto_generated_name);

                #[cfg(feature = "csv")]
                if csv {}

                if gzip_encoding {
                    rinex.to_gzip_file(path)?;
                } else {
                    rinex.to_file(path)?;
                }
            }
        },
        RinexType::MeteoData => {},
        RinexType::DORIS => {},
        rinex_type => panic!("`diff` does not apply to {}", rinex_type),
    }

    Ok(())
}
