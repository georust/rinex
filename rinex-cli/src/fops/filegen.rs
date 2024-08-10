use crate::cli::Context;
use crate::fops::{custom_prod_attributes, output_filename};
use crate::Error;
use clap::ArgMatches;
use rinex_qc::prelude::ProductType;

#[cfg(feature = "csv")]
use crate::fops::csv::write_obs_rinex as write_obs_rinex_csv;

/*
 * Dumps current context (usually preprocessed)
 * into either RINEX/SP3 format (maintaining consistent format) or CSV
 */
pub fn filegen(ctx: &Context, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
    #[cfg(feature = "csv")]
    if submatches.get_flag("csv") {
        write_csv(ctx, matches, submatches)?;
        return Ok(());
    }
    #[cfg(not(feature = "csv"))]
    if submatches.get_flag("csv") {
        panic!("Not available. Activate `csv` feature first.")
    }

    write(ctx, matches, submatches)?;
    Ok(())
}

fn write_csv(ctx: &Context, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    if let Some(rinex) = ctx_data.rinex(ProductType::Observation) {
        let prod = custom_prod_attributes(rinex, submatches);
        let output = output_filename(rinex, matches, submatches, prod);
        write_obs_rinex_csv(rinex, &output)?;
        info!("{} (OBS_RINEX) has been dumped", output);
    }
    Ok(())
}

fn write(ctx: &Context, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    for product in [
        ProductType::DORIS,
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::HighPrecisionOrbit,
        ProductType::IONEX,
        ProductType::ANTEX,
    ] {
        if let Some(rinex) = ctx_data.rinex(product) {
            let prod = custom_prod_attributes(rinex, submatches);
            let filename = output_filename(rinex, matches, submatches, prod);

            let output_path = ctx
                .workspace
                .root
                .join("OUTPUT")
                .join(filename)
                .to_string_lossy()
                .to_string();

            rinex.to_file(&output_path).unwrap_or_else(|_| {
                panic!("failed to generate {} RINEX \"{}\"", product, output_path)
            });

            info!("{} RINEX \"{}\" has been generated", product, output_path);
        }
    }
    Ok(())
}
