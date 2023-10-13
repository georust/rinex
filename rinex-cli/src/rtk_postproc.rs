use crate::cli::Cli;
use hifitime::Epoch;
use rinex::prelude::RnxContext;
use rtk::prelude::SolverEstimate;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub(crate) fn rtk_postproc(
    workspace: PathBuf,
    cli: &Cli,
    ctx: &RnxContext,
    results: HashMap<Epoch, SolverEstimate>,
) -> Result<(), std::io::Error> {
    let path = workspace.join("rtk.txt");
    let fullpath = path.to_string_lossy().to_string();
    let mut fd = File::create(&fullpath)?;
    let (x, y, z): (f64, f64, f64) = ctx
        .ground_position()
        .unwrap() // cannot fail
        .into();
    writeln!(
        fd,
        "Epoch, dx, dy, dz, x_ecef, y_ecef, z_ecef, hdop, vdop, rcvr_clock_bias, tdop"
    )?;
    for (epoch, estimate) in results {
        writeln!(
            fd,
            "{:?}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}, {:.6E}",
            epoch,
            estimate.dx,
            estimate.dy,
            estimate.dz,
            x + estimate.dx,
            y + estimate.dy,
            z + estimate.dz,
            estimate.hdop,
            estimate.vdop,
            estimate.dt,
            estimate.tdop
        )?;
    }
    info!("\"{}\" results generated", fullpath);
    Ok(())
}
