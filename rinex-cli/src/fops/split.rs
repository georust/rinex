use crate::cli::Context;
use crate::fops::custom_prod_attributes;
use crate::Error;
use clap::ArgMatches;
use rinex::prelude::{Epoch, Rinex};
use rinex_qc::prelude::{ProductType, Split};
use std::path::Path;

/// Dump and generate new [Rinex]
fn generate_rinex(
    ctx: &Context,
    product_id: ProductType,
    input_path: &Path,
    rinex: &Rinex,
    matches: &ArgMatches,
) -> Result<(), Error> {
    let prod = custom_prod_attributes(&rinex, matches);

    let extension = input_path
        .extension()
        .expect("failed to determine input file name")
        .to_string_lossy();

    let suffix = if extension.eq("gz") {
        Some(".gz")
    } else {
        None
    };

    let mut filename = rinex.standard_filename(false, suffix, Some(prod));

    // Can't we find a better output product name ?
    let t0 = rinex
        .first_epoch()
        .expect("failed to generate output file name");

    let (y, m, d, hh, mm, ss, _) = t0.to_gregorian_utc();

    filename.push_str(&format!("{}{}{}_{}{}{}", y, m, d, hh, mm, ss));

    let output = ctx.workspace.root.join(filename);

    rinex.to_file(&output)?;
    info!(
        "{} RINEX \"{}\" has been generated",
        product_id,
        output.to_string_lossy()
    );
    Ok(())
}

/// Split all input files (per [ProductType]) at specified [Epoch]
pub fn split(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;

    let t = matches
        .get_one::<Epoch>("split")
        .expect("split epoch is required");

    if let Some(sp3) = ctx_data.sp3_data() {
        let (sp3_a, sp3_b) = sp3.split(*t);
    }

    for product_id in [
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::IONEX,
    ] {
        if let Some(rinex) = ctx_data.get_rinex_data(product_id) {
            let (rinex_a, rinex_b) = rinex.split(*t);

            let input_path = ctx_data
                .files_iter(Some(product_id))
                .next()
                .expect("failed to determine output file name");

            generate_rinex(ctx, product_id, input_path, &rinex_a, &matches)?;
            generate_rinex(ctx, product_id, input_path, &rinex_b, &matches)?;
        }
    }

    Ok(())
}
