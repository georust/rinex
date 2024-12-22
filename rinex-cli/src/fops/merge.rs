use crate::Error;
use clap::ArgMatches;
use std::path::PathBuf;

use crate::{cli::Cli, preprocessing::preprocess_rinex};

#[cfg(feature = "csv")]
use crate::fops::csv::{
    write_meteo_rinex as csv_write_meteo_rinex,
    //  write_nav_rinex as csv_write_nav_rinex,
    write_obs_rinex as csv_write_obs_rinex,
};

use rinex_qc::prelude::{Merge, QcContext};

use rinex::prelude::{Rinex, RinexType};

/// Merges proposed (single) file and generates output results into the workspace
pub fn merge(ctx: &QcContext, cli: &Cli, matches: &ArgMatches) -> Result<(), Error> {
    let merge_path = matches.get_one::<PathBuf>("file").unwrap();

    let extension = merge_path
        .extension()
        .expect("failed to determine file extension")
        .to_string_lossy();

    let gzip_extension = extension.eq("gz");

    let gzip_encoding = cli.gzip_encoding();
    let short_rinex = cli.short_rinex_file_name();

    let rinex_b = if gzip_extension {
        Rinex::from_gzip_file(merge_path)
    } else {
        Rinex::from_file(merge_path)
    };

    let mut rinex_b = rinex_b.unwrap_or_else(|e| panic!("failed to parse B file: {}", e));

    // Apply same preprocessing pipeline
    preprocess_rinex(&mut rinex_b, &cli);

    let (name, rinex_c) = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let meta = ctx.obs_dataset.keys().collect::<Vec<_>>()[0];

            let rinex_a = ctx
                .obs_dataset
                .get(&meta)
                .unwrap_or_else(|| panic!("missing Observation RINex counterpart"));

            let rinex_c = rinex_a.merge(&rinex_b)?;
            (meta.name.clone(), rinex_c)
        },
        RinexType::NavigationData => {
            let rinex_a = ctx
                .nav_dataset
                .as_ref()
                .unwrap_or_else(|| panic!("missing Navigation RINex counterpart"));

            let file_stem = merge_path
                .file_stem()
                .expect("failed to determine output file name")
                .to_string_lossy()
                .to_string();

            let meta = file_stem.split('.').collect::<Vec<_>>()[0];

            ctx.create_subdir(&meta)
                .unwrap_or_else(|e| panic!("failed to create output directory: {}", e));

            let rinex_c = rinex_a.merge(&rinex_b)?;
            (meta.to_string(), rinex_c)
        },
        RinexType::MeteoData => {
            let meta = ctx.meteo_dataset.keys().collect::<Vec<_>>()[0];

            let rinex_a = ctx
                .meteo_dataset
                .get(&meta)
                .unwrap_or_else(|| panic!("missing Meteo RINex counterpart"));

            let rinex_c = rinex_a.merge(&rinex_b)?;
            (meta.name.clone(), rinex_c)
        },
        RinexType::IonosphereMaps => {
            let rinex_a = ctx
                .ionex_dataset
                .as_ref()
                .unwrap_or_else(|| panic!("missing IONex counterpart"));

            let file_stem = merge_path
                .file_stem()
                .expect("failed to determine output file name")
                .to_string_lossy()
                .to_string();

            let meta = file_stem.split('.').collect::<Vec<_>>()[0];

            ctx.create_subdir(&meta)
                .unwrap_or_else(|e| panic!("failed to create output directory: {}", e));

            let rinex_c = rinex_a.merge(&rinex_b)?;
            (meta.to_string(), rinex_c)
        },
        RinexType::ClockData => {
            panic!("cannot merge Clock RINex yet");
        },
        RinexType::AntennaData => {
            panic!("cannot merge ANTex yet");
        },
        RinexType::DORIS => {
            panic!("cannot merge DORIS yet");
        },
    };

    let suffix = if gzip_encoding { Some(".gz") } else { None };

    let auto_generated = rinex_c.standard_filename(short_rinex, suffix, None);

    let output_path = ctx.cfg.workspace.join(name).join(auto_generated);

    #[cfg(feature = "csv")]
    if matches.get_flag("csv") {
        match rinex_b.header.rinex_type {
            RinexType::ObservationData => {
                csv_write_obs_rinex(&rinex_c, &output_path)?;
                info!("\"{}\" has been generated", output_path.display());
                return Ok(());
            },
            RinexType::MeteoData => {
                csv_write_meteo_rinex(&rinex_c, &output_path)?;
                info!("\"{}\" has been generated", output_path.display());
                return Ok(());
            },
            rinex_type => {
                panic!("cannot format {} to csv yet", rinex_type);
            },
        }
    }

    if gzip_encoding {
        rinex_c.to_gzip_file(&output_path)?;
    } else {
        rinex_c.to_file(&output_path)?;
    }

    info!("\"{}\" has been generated", output_path.display());
    Ok(())
}
