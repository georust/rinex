mod cli;
use cli::Cli;

use log::{debug, info};

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    str::FromStr,
};

use flate2::{write::GzEncoder, Compression};
use thiserror::Error;

use rinex::{
    prelude::{binex::BIN2RNX, Duration},
    production::{Postponing, SnapshotMode},
};

// Supported output types
pub enum Output {
    // Simple file
    File(File),
    // Gzip compressed file
    GzipFile(GzEncoder<File>),
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::File(fd) => fd.write(buf),
            Self::GzipFile(fd) => fd.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(fd) => fd.flush(),
            Self::GzipFile(fd) => fd.flush(),
        }
    }
}

fn workspace(cli: &Cli) -> PathBuf {
    if let Some(workspace) = cli.workspace() {
        Path::new(workspace).to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("WORKSPACE")
    }
}

fn main() {
    let cli = Cli::new();

    let fp = cli.input_path();
    let mut fd = File::open(fp).unwrap_or_else(|e| panic!("failed to open file interface"));

    let mut bin2rnx = BIN2RNX::new_periodic(
        false,
        Duration::from_str("30s").unwrap(),
        Postponing::None,
        fd,
    );

    info!("RINEX collection starting");
    loop {
        println!("active: {}", bin2rnx.active);

        println!("collected: {:?}", bin2rnx.nav_rinex());

        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}
