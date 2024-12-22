//mod diff;
mod filegen;
mod merge;
mod split;
// mod tbin;

#[cfg(feature = "csv")]
pub mod csv;

//pub use diff::diff;
pub use filegen::filegen;
pub use merge::merge;
pub use split::split;
//pub use tbin::time_binning;

use clap::ArgMatches;

use rinex::{
    prelude::Rinex,
    prod::{DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU},
};

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
        if let Some(ref mut details) = opts.v3_details {
            details.country = country[..3].to_string();
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.country = country[..3].to_string();
            opts.v3_details = Some(default);
        }
    }
    if let Some(batch) = matches.get_one::<u8>("batch") {
        if let Some(ref mut details) = opts.v3_details {
            details.batch = *batch;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.batch = *batch;
            opts.v3_details = Some(default);
        }
    }
    if let Some(src) = matches.get_one::<DataSource>("source") {
        if let Some(ref mut details) = opts.v3_details {
            details.data_src = *src;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.data_src = *src;
            opts.v3_details = Some(default);
        }
    }
    if let Some(ppu) = matches.get_one::<PPU>("ppu") {
        if let Some(ref mut details) = opts.v3_details {
            details.ppu = *ppu;
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.ppu = *ppu;
            opts.v3_details = Some(default);
        }
    }
    if let Some(ffu) = matches.get_one::<FFU>("ffu") {
        if let Some(ref mut details) = opts.v3_details {
            details.ffu = Some(*ffu);
        } else {
            let mut default = DetailedProductionAttributes::default();
            default.ffu = Some(*ffu);
            opts.v3_details = Some(default);
        }
    }
    opts
}
