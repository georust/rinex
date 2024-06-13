use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;

use std::path::PathBuf;
use std::str::FromStr;

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

/*
 * Merges proposed (single) file and generates resulting output, into the workspace
 */
pub fn merge(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let merge_path = matches.get_one::<PathBuf>("file").unwrap();

    let merge_filepath = merge_path.to_string_lossy().to_string();

    let rinex_b = Rinex::from_file(&merge_filepath)?;

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx_data
                .observation()
                .ok_or(Error::MissingObservationRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::NavigationData => {
            let rinex_a = ctx_data
                .brdc_navigation()
                .ok_or(Error::MissingNavigationRinex)?;
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

/*
 * Time reframing: subdivide a RINEX into a batch of equal duration
 */
pub fn time_binning(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let duration = matches
        .get_one::<Duration>("interval")
        .expect("duration is required");

    if *duration == Duration::ZERO {
        panic!("invalid (null) duration");
    }

    for product in [
        ProductType::IONEX,
        ProductType::DORIS,
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::HighPrecisionOrbit,
    ] {
        // input data determination
        if let Some(rinex) = ctx_data.rinex(product) {
            // time framing determination
            let (mut first, end) = (
                rinex
                    .first_epoch()
                    .expect("failed to determine first epoch"),
                rinex.last_epoch().expect("failed to determine last epoch"),
            );

            let mut last = first + *duration;

            // production attributes: initialize Batch counter
            let mut prod = custom_prod_attributes(rinex, matches);
            if let Some(ref mut details) = prod.details {
                details.batch = 0_u8;
            } else {
                let mut details = DetailedProductionAttributes::default();
                details.batch = 0_u8;
                prod.details = Some(details);
            };

            // run time binning algorithm
            while last <= end {
                let rinex = rinex
                    .filter(&Filter::from_str(&format!("< {:?}", last)).unwrap())
                    .filter(&Filter::from_str(&format!(">= {:?}", first)).unwrap());

                // generate standardized name
                let filename = output_filename(&rinex, matches, prod.clone());

                let output = ctx
                    .workspace
                    .root
                    .join("OUTPUT")
                    .join(&filename)
                    .to_string_lossy()
                    .to_string();

                rinex.to_file(&output)?;
                info!("{} RINEX \"{}\" has been generated", product, output);

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

/*
 * Substract RINEX[A]-RINEX[B]
 */
pub fn diff(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let ctx_data = &ctx.data;
    let path_a = ctx_data
        .files(ProductType::Observation)
        .expect("failed to determine output file name")
        .first()
        .unwrap();

    let path_b = matches.get_one::<PathBuf>("file").unwrap();

    let path_b = path_b.to_string_lossy().to_string();
    let rinex_b = Rinex::from_file(&path_b)
        .unwrap_or_else(|_| panic!("failed to load {}: invalid RINEX", path_b));

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx_data
                .observation()
                .expect("RINEX (A) - (B) requires OBS RINEX files");

            //TODO: change this to crnx2rnx_mut()
            rinex_a.crnx2rnx().substract(&rinex_b.crnx2rnx())
        },
        t => panic!("operation not feasible for {}", t),
    };

    let mut extension = String::new();

    let filename = path_a
        .file_stem()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    if filename.contains('.') {
        /* .crx.gz case */
        let mut iter = filename.split('.');
        let _filename = iter
            .next()
            .expect("failed to determine output file name")
            .to_string();
        extension.push_str(iter.next().expect("failed to determine output file name"));
        extension.push('.');
    }

    let file_ext = path_a
        .extension()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    extension.push_str(&file_ext);

    let fullpath = ctx
        .workspace
        .root
        .join(format!("DIFFERENCED.{}", extension))
        .to_string_lossy()
        .to_string();

    rinex_c.to_file(&fullpath)?;

    info!("OBS RINEX \"{}\" has been generated", fullpath);
    Ok(())
}
