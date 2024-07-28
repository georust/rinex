//! helpers to export to CSV if desired,
//! and not only generate HTML plots.

use crate::cli::Workspace;
use hifitime::Epoch;
use std::fs::File;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to write csv file")]
    IoError(#[from] std::io::Error),
}

/// Custom CSV report
pub struct CSV {
    fd: File,
}

impl CSV {
    /// Creates new CSV report in this session.
    /// Panics on write permission issues.
    pub fn new(
        workspace: &Workspace,
        filename: &str,
        title: &str,
        labels: &str,
    ) -> Result<Self, Error> {
        let mut fd = workspace.create_file(filename);
        writeln!(fd, "================================================")?;
        writeln!(fd, "title  : {}", title)?;
        writeln!(fd, "labels : {}", labels)?;
        writeln!(
            fd,
            "version: rinex-cli v{} - https://georust.org",
            env!("CARGO_PKG_VERSION")
        )?;
        writeln!(fd, "================================================")?;
        Ok(Self { fd })
    }
    /// Report timedomain data as CSV
    pub fn export_timedomain<T: std::fmt::UpperExp>(
        &mut self,
        x: &Vec<Epoch>,
        y: &Vec<T>,
    ) -> Result<(), Error> {
        for (x, y) in x.iter().zip(y.iter()) {
            writeln!(self.fd, "{:?}, {:.6E}", x, y)?;
        }
        Ok(())
    }
}
