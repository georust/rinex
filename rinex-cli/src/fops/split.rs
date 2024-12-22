use crate::Error;
use clap::ArgMatches;

use crate::cli::Cli;
use rinex::prelude::{Epoch, Rinex, RinexType};
use rinex_qc::prelude::{QcContext, Split};

#[cfg(feature = "csv")]
use crate::fops::csv::{
    write_meteo_rinex as csv_write_meteo_rinex,
    // write_nav_rinex as csv_write_nav_rinex,
    write_obs_rinex as csv_write_obs_rinex,
};

/// Dump and generate new [Rinex]
fn generate_dual_rinex(
    ctx: &QcContext,
    rinex_a: &Rinex,
    rinex_b: &Rinex,
    csv: bool,
    short_filename: bool,
    gzip_encoding: bool,
    subdir: Option<String>,
) -> Result<(), Error> {
    if let Some(subdir) = &subdir {
        ctx.create_subdir(&subdir)
            .unwrap_or_else(|e| panic!("failed to generate output dir: {}", e));
    }

    let suffix = if gzip_encoding { Some(".gz") } else { None };

    let name_a = rinex_a.standard_filename(short_filename, suffix, None);

    let path_a = if let Some(subdir) = &subdir {
        ctx.cfg.workspace.join(subdir).join(name_a)
    } else {
        ctx.cfg.workspace.join(name_a)
    };

    if gzip_encoding {
        rinex_a.to_gzip_file(&path_a)?;
    } else {
        rinex_a.to_file(&path_a)?;
    }

    let name_b = rinex_b.standard_filename(short_filename, suffix, None);

    let path_b = if let Some(subdir) = &subdir {
        ctx.cfg.workspace.join(subdir).join(name_b)
    } else {
        ctx.cfg.workspace.join(name_b)
    };

    #[cfg(feature = "csv")]
    if csv {
        match rinex_a.header.rinex_type {
            RinexType::ObservationData => {
                csv_write_obs_rinex(&rinex_a, &path_a)
                    .unwrap_or_else(|e| panic!("failed to dump as csv: {}", e));

                csv_write_obs_rinex(&rinex_b, &path_b)
                    .unwrap_or_else(|e| panic!("failed to dump as csv: {}", e));
            },
            RinexType::MeteoData => {},
            RinexType::NavigationData => {},
            rinex_type => {
                panic!("cannot format {} to CSV yet", rinex_type);
            },
        }
        return Ok(());
    }

    if gzip_encoding {
        rinex_b.to_gzip_file(&path_b)?;
    } else {
        rinex_b.to_file(&path_b)?;
    }

    Ok(())
}

/// Split all input files (per [ProductType]) at specified [Epoch]
pub fn split(ctx: &QcContext, cli: &Cli, matches: &ArgMatches) -> Result<(), Error> {
    let t = matches
        .get_one::<Epoch>("split")
        .expect("split epoch is required");

    let csv = matches.get_flag("csv");
    let short_rinex = cli.short_rinex_file_name();
    let gzip_encoding = cli.gzip_encoding();

    // apply to all internal products
    for (meta, rinex) in &ctx.obs_dataset {
        ctx.create_subdir(&meta.name)
            .unwrap_or_else(|e| panic!("failed to generate output dir: {}", e));

        let (rinex_a, rinex_b) = rinex.split(*t);

        generate_dual_rinex(
            ctx,
            &rinex_a,
            &rinex_b,
            csv,
            short_rinex,
            gzip_encoding,
            Some(meta.name.clone()),
        )
        .unwrap_or_else(|e| panic!("file synthesis error: {}", e));
    }

    if let Some(rinex) = &ctx.nav_dataset {
        let (rinex_a, rinex_b) = rinex.split(*t);

        generate_dual_rinex(
            ctx,
            &rinex_a,
            &rinex_b,
            csv,
            short_rinex,
            gzip_encoding,
            None,
        )
        .unwrap_or_else(|e| panic!("file synthesis error: {}", e));
    }

    if let Some(ionex) = &ctx.ionex_dataset {
        let (rinex_a, rinex_b) = ionex.split(*t);

        generate_dual_rinex(
            ctx,
            &rinex_a,
            &rinex_b,
            csv,
            short_rinex,
            gzip_encoding,
            None,
        )
        .unwrap_or_else(|e| panic!("file synthesis error: {}", e));
    }

    Ok(())
}
