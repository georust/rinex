use crate::Error;
use clap::ArgMatches;

use rinex_qc::prelude::QcContext;

#[cfg(feature = "csv")]
use crate::fops::csv::{
    write_meteo_rinex as write_meteo_rinex_csv,
    // write_nav_rinex as write_nav_rinex_csv,
    // write_sp3 as write_sp3_csv,
    write_obs_rinex as write_obs_rinex_csv,
};

/// Dump current context (possibly preprocessed) data context
/// into encountered formats (all formats preserved)
/// For example: RINEX will produce RINEX.
/// --csv export exists.
/// The --rnx2crx and --crx2rnx may have already been applied and
/// are naturally taken into account.
pub fn filegen(
    ctx: &QcContext,
    matches: &ArgMatches,
    submatches: &ArgMatches,
) -> Result<(), Error> {
    #[cfg(feature = "csv")]
    if submatches.get_flag("csv") {
        write_csv(ctx, matches, submatches)?;
        return Ok(());
    }

    #[cfg(not(feature = "csv"))]
    if submatches.get_flag("csv") {
        panic!("Not available. Compile with `csv` feature.")
    }

    write(ctx, matches, submatches)?;
    Ok(())
}

//#[cfg(feature = "csv")]
fn write_csv(ctx: &QcContext, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
    for (meta, rinex) in ctx.obs_dataset.iter() {
        let auto_generated_name = rinex.standard_filename(false, Some(".csv"), None);
        let fullpath = ctx.cfg.workspace.join(&meta.name).join(auto_generated_name);
        write_obs_rinex_csv(rinex, &fullpath)?;
        info!("OBSERVATION RINex \"{}\" dumped as csv", meta.name);
    }

    for (meta, rinex) in ctx.meteo_dataset.iter() {
        let auto_generated_name = rinex.standard_filename(false, Some(".csv"), None);
        let fullpath = ctx.cfg.workspace.join(&meta.name).join(auto_generated_name);
        write_meteo_rinex_csv(rinex, &fullpath)?;
        info!("METEO RINex \"{}\" dumped as csv", meta.name);
    }

    if let Some(_nav) = &ctx.nav_dataset {
        //let auto_generated_name = nav.standard_filename(false, Some(".csv"), None);
        //let fullpath = ctx.cfg.workspace.join(&auto_generated_name);
        //write_nav_rinex_csv(rinex, &fullpath)?;
        //info!("NAV RINex \"{}\" dumped as csv", auto_generated_name);
    }

    Ok(())
}

fn write(ctx: &QcContext, matches: &ArgMatches, submatches: &ArgMatches) -> Result<(), Error> {
    // OBS RINex
    for (meta, rinex) in ctx.obs_dataset.iter() {
        let auto_generated_name = rinex.standard_filename(false, None, None);
        let fullpath = ctx.cfg.workspace.join(&meta.name).join(auto_generated_name);

        rinex
            .to_file(&fullpath)
            .unwrap_or_else(|_| panic!("failed to generate OBSERVATION \"{}\"", meta.name));

        info!("OBSERVATION RINex \"{}\" has been generated", meta.name);
    }

    // METEO RINex
    for (meta, rinex) in ctx.meteo_dataset.iter() {
        let auto_generated_name = rinex.standard_filename(false, None, None);
        let fullpath = ctx.cfg.workspace.join(&meta.name).join(auto_generated_name);

        rinex
            .to_file(&fullpath)
            .unwrap_or_else(|_| panic!("failed to generate OBSERVATION \"{}\"", meta.name));

        info!("METEO RINex \"{}\" has been generated", meta.name);
    }

    Ok(())
}
