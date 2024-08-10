use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::Epoch;
use rinex::Split;
use rinex_qc::prelude::{ProductType};

/*
 * Splits input files at specified Time Instant
 */
pub fn split(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let split_instant = matches
        .get_one::<Epoch>("split")
        .expect("split epoch is required");

    for product in [
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::IONEX,
    ] {
        if let Some(rinex) = ctx_data.rinex(product) {
            let (rinex_a, rinex_b) = rinex
                .split(*split_instant)
                .unwrap_or_else(|e| panic!("failed to split {} RINEX: {}", product, e));

            let first_epoch = rinex_a
                .first_epoch()
                .unwrap_or_else(|| panic!("failed to determine {} file suffix", product));

            let (y, m, d, hh, mm, ss, _) = first_epoch.to_gregorian_utc();
            let file_suffix = format!(
                "{}{}{}_{}{}{}{}",
                y, m, d, hh, mm, ss, first_epoch.time_scale
            );

            let path = ctx_data
                .files(product)
                .unwrap_or_else(|| panic!("failed to determine output {} filename", product))
                .first()
                .unwrap();

            let filename = path
                .file_stem()
                .unwrap_or_else(|| panic!("failed to determine output {} filename", product))
                .to_string_lossy()
                .to_string();

            let mut extension = String::new();

            let filename = if filename.contains('.') {
                /* .crx.gz case */
                let mut iter = filename.split('.');
                let filename = iter
                    .next()
                    .expect("failed to determine output file name")
                    .to_string();
                extension.push_str(iter.next().expect("failed to determine output file name"));
                extension.push('.');
                filename
            } else {
                filename.clone()
            };

            let file_ext = path
                .extension()
                .expect("failed to determine output file name")
                .to_string_lossy()
                .to_string();

            extension.push_str(&file_ext);

            let output = ctx
                .workspace
                .root
                .join(format!("{}-{}.{}", filename, file_suffix, extension))
                .to_string_lossy()
                .to_string();

            rinex_a.to_file(&output)?;
            info!("\"{}\" has been generated", output);

            let first_epoch = rinex_b
                .first_epoch()
                .expect("failed to determine file suffix");

            let (y, m, d, hh, mm, ss, _) = first_epoch.to_gregorian_utc();
            let file_suffix = format!(
                "{}{}{}_{}{}{}{}",
                y, m, d, hh, mm, ss, first_epoch.time_scale
            );

            let path = ctx_data
                .files(product)
                .unwrap_or_else(|| panic!("failed to determine output {} filename", product))
                .first()
                .unwrap();

            let filename = path
                .file_stem()
                .expect("failed to determine output file name")
                .to_string_lossy()
                .to_string();

            let output = ctx
                .workspace
                .root
                .join(format!("{}-{}.{}", filename, file_suffix, extension))
                .to_string_lossy()
                .to_string();

            rinex_b.to_file(&output)?;
            info!("{} RINEX \"{}\" has been generated", product, output);
        }
    }
    Ok(())
}
