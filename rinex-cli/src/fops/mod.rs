mod diff;
mod merge;
mod split;
mod tbin;

pub use diff::diff;
pub use merge::merge;
pub use split::split;
pub use tbin::time_binning;

use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;

use std::path::PathBuf;
//use std::str::FromStr;

use rinex::{
    prelude::{Duration, Epoch, Rinex, RinexType},
    prod::{DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU},
    Merge, Split,
};

use rinex_qc::prelude::{Filter, Preprocessing, ProductType};

/*
 * Parses share RINEX production attributes.
 * This helps accurate file production,
 * and also allows customization from files that did not originally follow
 * standard naming conventions
 */
fn custom_prod_attributes(rinex: &Rinex, matches: &ArgMatches) -> ProductionAttributes {
    // Start from smartly guessed attributes and replace
    // manually customized fields
    let mut opts = rinex.guess_production_attributes();
    if let Some(agency) = matches.get_one::<String>("agency") {
        opts.name = agency.to_string();
    }
    if let Some(country) = matches.get_one::<String>("country") {
        if let Some(ref mut details) = opts.details {
            details.country = country[..3].to_string();
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.country = country[..3].to_string();
            opts.details = Some(default);
        }
    }
    if let Some(batch) = matches.get_one::<u8>("batch") {
        if let Some(ref mut details) = opts.details {
            details.batch = *batch;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.batch = *batch;
            opts.details = Some(default);
        }
    }
    if let Some(src) = matches.get_one::<DataSource>("source") {
        if let Some(ref mut details) = opts.details {
            details.data_src = *src;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.data_src = *src;
            opts.details = Some(default);
        }
    }
    if let Some(ppu) = matches.get_one::<PPU>("ppu") {
        if let Some(ref mut details) = opts.details {
            details.ppu = *ppu;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.ppu = *ppu;
            opts.details = Some(default);
        }
    }
    if let Some(ffu) = matches.get_one::<FFU>("ffu") {
        if let Some(ref mut details) = opts.details {
            details.ffu = Some(*ffu);
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.ffu = Some(*ffu);
            opts.details = Some(default);
        }
    }
    opts
}

/*
 * Returns output filename to be generated, for this kind of Product
 * TODO: some customization might impact the Header section
 *       that we should slightly rework, to be 100% correct
 */
fn output_filename(rinex: &Rinex, matches: &ArgMatches, prod: ProductionAttributes) -> String {
    // Parse possible custom opts
    let short = matches.get_flag("short");
    let gzip = if matches.get_flag("gzip") {
        Some(".gz")
    } else {
        None
    };

    debug!("{:?}", prod);

    // Use smart determination
    rinex.standard_filename(short, gzip, Some(prod))
}

/*
 * Dumps current context (usually preprocessed)
 * into RINEX format maintaining consistent format
 */
pub fn filegen(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
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
            let filename = output_filename(rinex, matches, prod);

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
