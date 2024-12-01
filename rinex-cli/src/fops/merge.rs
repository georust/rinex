use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::{Rinex, RinexType};
use std::path::PathBuf;

use rinex_qc::prelude::Merge;

/// Merges proposed (single) file and generates output results into the workspace
pub fn merge(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let merge_path = matches.get_one::<PathBuf>("file").unwrap();

    let merge_filepath = merge_path.to_string_lossy().to_string();

    let rinex_b = Rinex::from_file(&merge_filepath)?;

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx_data
                .observation_data()
                .ok_or(Error::MissingObservationRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::NavigationData => {
            let rinex_a = ctx_data
                .brdc_navigation_data()
                .ok_or(Error::MissingNavigationRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::MeteoData => {
            let rinex_a = ctx_data.meteo_data().ok_or(Error::MissingMeteoRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::IonosphereMaps => {
            let rinex_a = ctx_data.ionex_data().ok_or(Error::MissingIONEX)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::ClockData => {
            let rinex_a = ctx_data.clock_data().ok_or(Error::MissingClockRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        _ => unimplemented!(),
    };

    let suffix = merge_path
        .file_name()
        .expect("failed to determine output path")
        .to_string_lossy()
        .to_string();

    let output_path = ctx
        .workspace
        .root
        .join(suffix)
        .to_string_lossy()
        .to_string();

    rinex_c.to_file(&output_path)?;

    info!("\"{}\" has been generated", output_path);
    Ok(())
}
