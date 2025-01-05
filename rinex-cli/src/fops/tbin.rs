use clap::ArgMatches;
use rinex::prelude::Duration;
use rinex_qc_traits::Split;

use crate::{cli::Cli, Error};

use rinex_qc::prelude::QcContext;

/// Time binning (batch split of equal duration) file operation
pub fn tbin(ctx: &QcContext, cli: &Cli, submatches: &ArgMatches) -> Result<(), Error> {
    // parse fixed duration
    let dt = submatches
        .get_one::<Duration>("interval")
        .expect("duration is required");

    if *dt == Duration::ZERO {
        panic!("invalid (null) duration");
    }

    let short_name = cli.short_rinex_file_name();
    let gzip_encoding = cli.gzip_encoding();

    let suffix = if gzip_encoding { Some(".gz") } else { None };

    // tbin applies to any temporal format
    // 1. prepare output
    for (obs_meta, _) in &ctx.obs_dataset {
        ctx.create_subdir(&obs_meta.meta.name)
            .unwrap_or_else(|e| panic!("failed to generate output dir: {}", e));
    }
    for (meta, _) in &ctx.meteo_dataset {
        ctx.create_subdir(&meta.name)
            .unwrap_or_else(|e| panic!("failed to generate output dir: {}", e));
    }

    for (obs_meta, rinex) in &ctx.obs_dataset {
        for split in rinex.split_even_dt(*dt).iter() {
            let auto_generated_name = split.standard_filename(short_name, suffix, None);
            let path = ctx
                .cfg
                .workspace
                .join(&obs_meta.meta.name)
                .join(auto_generated_name);
            if gzip_encoding {
                split.to_gzip_file(path)?;
            } else {
                split.to_file(path)?;
            }
        }
    }

    for (meta, rinex) in &ctx.meteo_dataset {
        for split in rinex.split_even_dt(*dt).iter() {
            let auto_generated_name = split.standard_filename(short_name, suffix, None);
            let path = ctx.cfg.workspace.join(&meta.name).join(auto_generated_name);
            if gzip_encoding {
                split.to_gzip_file(path)?;
            } else {
                split.to_file(path)?;
            }
        }
    }

    if let Some(nav) = &ctx.nav_dataset {
        for split in nav.split_even_dt(*dt).iter() {
            let auto_generated_name = split.standard_filename(short_name, suffix, None);
            let path = ctx.cfg.workspace.join(auto_generated_name);
            if gzip_encoding {
                split.to_gzip_file(path)?;
            } else {
                split.to_file(path)?;
            }
        }
    }

    if let Some(ionex) = &ctx.ionex_dataset {
        for split in ionex.split_even_dt(*dt).iter() {
            let auto_generated_name = split.standard_filename(short_name, suffix, None);
            let path = ctx.cfg.workspace.join(auto_generated_name);
            if gzip_encoding {
                split.to_gzip_file(path)?;
            } else {
                split.to_file(path)?;
            }
        }
    }

    Ok(())
}
