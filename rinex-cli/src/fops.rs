use crate::cli::Context;
use crate::Error;
use clap::ArgMatches;

use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

use rinex::{
    prelude::{Duration, Epoch, ProductType, Rinex, RinexType},
    preprocessing::*,
    Merge, Split,
};

/*
 * Dumps current context (usually preprocessed)
 * into RINEX format maintaining consistent format
 */
pub fn filegen(ctx: &Context, _matches: &ArgMatches) -> Result<(), Error> {
    for product in [
        Observation,
        MeteoObservation,
        BroadcastNavigation,
        HighPrecisionClock,
        Ionex,
        Antex,
    ] {
        if let Some(rinex) = ctx.data.rinex(product) {
            let filename = ctx
                .data
                .files(product)
                .expect(&format!("failed to determine {} output", product))
                .get(0)
                .expect(&format!("failed to determine {} output", product))
                .file_name()
                .expect(&format!("failed to determine {} output", product))
                .to_string_lossy()
                .to_string();

            let output_path = ctx.workspace.join(filename).to_string_lossy().to_string();

            rinex.to_file(&output_path).unwrap_or_else(|_| {
                panic!("failed to generate rinex observations \"{}\"", output_path)
            });

            info!("{} RINEX \"{}\" has been generated", output_path);
        }
    }
    Ok(())
}

/*
 * Merges proposed (single) file and generates resulting output, into the workspace
 */
pub fn merge(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let merge_path = matches.get_one::<PathBuf>("file").unwrap();

    let merge_filepath = merge_path.to_string_lossy().to_string();

    let rinex_b = Rinex::from_file(&merge_filepath)?;

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx
                .data
                .observation()
                .ok_or(Error::MissingObservationRinex)?;
            rinex_a.merge(&rinex_b)?
        },
        RinexType::NavigationData => {
            let rinex_a = ctx
                .data
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

    let output_path = ctx.workspace.join(suffix).to_string_lossy().to_string();

    rinex_c.to_file(&output_path)?;

    info!("\"{}\" has been generated", output_path);
    Ok(())
}

/*
 * Splits input files at specified Time Instant
 */
pub fn split(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let split_instant = matches
        .get_one::<Epoch>("split")
        .expect("split epoch is required");

    for product in [
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::Ionex,
    ] {
        if let Some(rinex) = ctx.data.rinex(product) {
            let (rinex_a, rinex_b) = rinex
                .split(*split_instant)
                .unwrap_or_else(|e| panic!("failed to split {} RINEX: {}", product, e));

            let first_epoch = rinex_a
                .first_epoch()
                .expect(&format!("failed to determine {} file suffix", product));

            let (y, m, d, hh, mm, ss, _) = first_epoch.to_gregorian_utc();
            let file_suffix = format!(
                "{}{}{}_{}{}{}{}",
                y, m, d, hh, mm, ss, first_epoch.time_scale
            );

            let path = ctx
                .data
                .files(product)
                .expect(&format!("failed to determine output {} filename", product))
                .get(0)
                .unwrap();

            let filename = path
                .file_stem()
                .expect(&format!("failed to determine output {} filename", product))
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

            let path = ctx
                .data
                .files(product)
                .expect(&format!("failed to determine output {} filename", product))
                .get(0)
                .unwrap();

            let filename = path
                .file_stem()
                .expect("failed to determine output file name")
                .to_string_lossy()
                .to_string();

            let output = ctx
                .workspace
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
 * Time reframing: subdivde a RINEX into a batch of equal duration
 */
pub fn time_binning(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let duration = matches
        .get_one::<Duration>("interval")
        .expect("duration is required");

    if *duration == Duration::ZERO {
        panic!("invalid duration");
    }

    for product in [
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::Ionex,
    ] {
        // input data determination
        if let Some(rinex) = ctx.data.rinex(product) {
            // time framing determination
            let (mut first, end) = (
                rinex
                    .first_epoch()
                    .expect("failed to determine first epoch"),
                rinex.last_epoch().expect("failed to determine last epoch"),
            );

            let mut last = first + *duration;

            // filename determination
            let data_path = ctx
                .data
                .files(product)
                .unwrap()
                .get(0)
                .expect(&format!("failed to determine output {} file name", product));

            let filename = data_path
                .file_stem()
                .expect(&format!("failed to determine output {} file name", product))
                .to_string_lossy()
                .to_string();

            let mut extension = String::new();

            let filename = if filename.contains('.') {
                /* .crx.gz case */
                let mut iter = filename.split('.');
                let filename = iter
                    .next()
                    .expect(&format!("failed to determine output {} file name", product))
                    .to_string();
                extension.push_str(
                    iter.next()
                        .expect(&format!("failed to determine output {} file name", product)),
                );
                extension.push('.');
                filename
            } else {
                filename.clone()
            };

            let file_ext = data_path
                .extension()
                .expect(&format!("failed to determine output {} file name", product))
                .to_string_lossy()
                .to_string();

            extension.push_str(&file_ext);

            // run time binning algorithm
            while last <= end {
                let rinex = rinex
                    .filter(Filter::from_str(&format!("< {:?}", last)).unwrap())
                    .filter(Filter::from_str(&format!(">= {:?}", first)).unwrap());

                let (y, m, d, hh, mm, ss, _) = first.to_gregorian_utc();
                let file_suffix = format!("{}{}{}_{}{}{}{}", y, m, d, hh, mm, ss, first.time_scale);

                let output = ctx
                    .workspace
                    .join(&format!("{}-{}.{}", filename, file_suffix, extension))
                    .to_string_lossy()
                    .to_string();

                rinex.to_file(&output)?;
                info!("{} RINEX \"{}\" has been generated", product, output);

                first += *duration;
                last += *duration;
            }
        }
    }
    Ok(())
}

/*
 * Substract RINEX[A]-RINEX[B]
 */
pub fn substract(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    let path_b = matches.get_one::<PathBuf>("file").unwrap();

    let path_b = path_b.to_string_lossy().to_string();
    let rinex_b = Rinex::from_file(&path_b)
        .unwrap_or_else(|_| panic!("failed to load {}: invalid RINEX", path_b));

    let rinex_c = match rinex_b.header.rinex_type {
        RinexType::ObservationData => {
            let rinex_a = ctx
                .data
                .observation_mut()
                .expect("RINEX (A) - (B) requires OBS RINEX files");

            rinex_a.crnx2rnx().substract(&rinex_b.crnx2rnx())
        },
        t => panic!("operation not feasible for {}", t),
    };

    let mut extension = String::new();

    let path = ctx
        .data
        .files(ProductType::Observation)
        .expect("failed to determine output file name")
        .get(0)
        .unwrap();

    let filename = path
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

    let file_ext = path
        .extension()
        .expect("failed to determine output file name")
        .to_string_lossy()
        .to_string();

    extension.push_str(&file_ext);

    let fullpath = ctx
        .workspace
        .join(format!("DIFFERENCED.{}", extension))
        .to_string_lossy()
        .to_string();

    rinex_c.to_file(&fullpath)?;

    info!("OBS RINEX \"{}\" has been generated", fullpath);
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn open_with_web_browser(path: &str) {
    let web_browsers = vec!["firefox", "chromium"];
    for browser in web_browsers {
        let child = Command::new(browser).args([path]).spawn();
        if child.is_ok() {
            return;
        }
    }
}

#[cfg(target_os = "macos")]
pub fn open_with_web_browser(path: &str) {
    Command::new("open")
        .args(&[path])
        .output()
        .expect("open() failed, can't open HTML content automatically");
}

#[cfg(target_os = "windows")]
pub fn open_with_web_browser(path: &str) {
    Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"start {}"#, path))
        .output()
        .expect("failed to open generated HTML content");
}
