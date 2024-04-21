//! helpers to export to CSV if desired,
//! and not only generate HTML plots.

use hifitime::Epoch;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to write csv file")]
    IoError(#[from] std::io::Error),
}

/*
 * Use this to export Time domain plots (most widely used plot type)
 */
pub fn csv_export_timedomain<T: std::fmt::UpperExp>(
    path: &Path,
    title: &str,
    labels: &str,
    x: &[Epoch],
    y: &[T],
) -> Result<(), Error> {
    let mut fd = File::create(path)?;
    writeln!(fd, "================================================")?;
    writeln!(fd, "title  : {}", title)?;
    writeln!(fd, "labels : {}", labels)?;
    writeln!(
        fd,
        "version: rinex-cli v{} - https://georust.org",
        env!("CARGO_PKG_VERSION")
    )?;
    writeln!(fd, "================================================")?;
    for (x, y) in x.iter().zip(y.iter()) {
        writeln!(fd, "{:?}, {:.6E}", x, y)?;
    }
    writeln!(fd, "================================================")?;
    Ok(())
}
