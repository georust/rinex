use crate::fops::{output_filename, custom_prod_attributes};
use rinex_qc::prelude::{ProductType};
use rinex::prod::DetailedProductionAttributes;
use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;

/*
 * Dumps current context (usually preprocessed)
 * into either RINEX/SP3 format (maintaining consistent format) or CSV
 */
pub fn filegen(ctx: &Context, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
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
            let prod = custom_prod_attributes(rinex, matches);
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
