use crate::cli::Context;
use crate::fops::custom_prod_attributes;
use crate::fops::output_filename;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::{Duration, RinexType};
use rinex::prod::DetailedProductionAttributes;
use rinex_qc::prelude::{Filter, Preprocessing};

/// Time binning (batch split of equal duration) file operation
pub fn time_binning(
    ctx: &Context,
    matches: &ArgMatches,
    submatches: &ArgMatches,
) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let duration = matches
        .get_one::<Duration>("interval")
        .expect("duration is required");

    if *duration == Duration::ZERO {
        panic!("invalid (null) duration");
    }

    // RINEX time binning
    for (rinex_type, dir) in [
        (RinexType::IonosphereMaps, "IONEX"),
        (RinexType::DORIS, "DORIS"),
        (RinexType::ObservationData, "OBSERVATIONS"),
        (RinexType::MeteoData, "METEO"),
        (RinexType::NavigationData, "BRDC"),
        (RinexType::ClockData, "CLOCK"),
    ] {
        // input data determination

        if let Some(rinex) = ctx_data.get_rinex_data(rinex_type) {
            // create work dir
            ctx.workspace.create_subdir(dir);

            // time frame determination
            let (mut first, end) = (
                rinex
                    .first_epoch()
                    .expect("failed to determine first epoch"),
                rinex.last_epoch().expect("failed to determine last epoch"),
            );

            let mut last = first + *duration;

            // production attributes: initialize Batch counter
            let mut prod = custom_prod_attributes(rinex, submatches);
            if let Some(ref mut details) = prod.details {
                details.batch = 0_u8;
            } else {
                let mut details = DetailedProductionAttributes::default();
                details.batch = 0_u8;
                prod.details = Some(details);
            };

            // run time binning algorithm
            while last <= end {
                let lower = Filter::lower_than(&last.to_string()).unwrap();
                let greater = Filter::greater_equals(&first.to_string()).unwrap();

                debug!("batch: {} < {}", first, last);
                let batch = rinex.filter(&lower).filter(&greater);

                // generate standardized name
                let filename = output_filename(&batch, matches, submatches, prod.clone());

                let output = ctx
                    .workspace
                    .root
                    .join("OUTPUT")
                    .join(&filename)
                    .to_string_lossy()
                    .to_string();

                batch.to_file(&output)?;
                info!("\"{}\" ({}) has been generated", output, rinex_type);

                first += *duration;
                last += *duration;

                if let Some(ref mut details) = prod.details {
                    details.batch += 1;
                }
            }
        }
    }
    Ok(())
}
