mod cli;
use cli::Cli;

use log::{debug, info};

use rinex::prelude::{binex::RNX2BIN, FormattingError, ParsingError, Rinex};

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use flate2::{write::GzEncoder, Compression};
use thiserror::Error;

/// Supported output types
pub enum Output {
    // Simple file
    File(File),
    // Gzip compressed file
    GzipFile(GzEncoder<File>),
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(0)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::File(fd) => fd.flush(),
            Self::GzipFile(fd) => fd.flush(),
        }
    }
}

impl Output {
    pub fn new(
        rinex: &Rinex,
        gzip_in: bool,
        workspace: &Path,
        gzip_out: bool,
        short_name: bool,
        custom_name: Option<&String>,
    ) -> Self {
        if let Some(custom) = custom_name {
            let path = workspace.join(custom);

            let mut fd = File::create(&path)
                .unwrap_or_else(|e| panic!("Failed to create file within workspace"));

            if gzip_in || gzip_out {
                info!("Generating custom gzip file: {}", path.display());
                let mut fd = GzEncoder::new(fd, Compression::new(5));
                Output::GzipFile(fd)
            } else {
                info!("Generating custom file: {}", path.display());
                Output::File(fd)
            }
        } else {
            // auto generated name
            let mut suffix = ".bin".to_string();

            if gzip_in || gzip_out {
                suffix.push_str(".gz");
            }

            let auto = rinex.standard_filename(short_name, Some(&suffix), None);

            let path = workspace.join(auto);

            let mut fd = File::create(&path)
                .unwrap_or_else(|e| panic!("Failed to create file within workspace"));

            if gzip_in || gzip_out {
                info!("Generating gzip file: {}", path.display());
                let mut fd = GzEncoder::new(fd, Compression::new(5));
                Output::GzipFile(fd)
            } else {
                info!("Generating file: {}", path.display());
                Output::File(fd)
            }
        }
    }
}

#[derive(Debug, Error)]
enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("formatting error")]
    FormattingError(#[from] FormattingError),
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

fn binex_streaming(output: &mut Output, rnx2bin: &mut RNX2BIN) {
    const BUF_SIZE: usize = 4096;
    let mut buf = [0; BUF_SIZE];
    debug!("RNX2BIN streaming!");
    loop {
        match rnx2bin.next() {
            Some(msg) => {
                msg.encode(&mut buf, BUF_SIZE)
                    .unwrap_or_else(|e| panic!("BINEX encoding error: {:?}", e));

                output
                    .write(&buf)
                    .unwrap_or_else(|e| panic!("I/O error: {}", e));

                buf = [0; BUF_SIZE];
            },
            None => {},
        }
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::new();

    let meta = cli.meta();
    let input_path = cli.input_path();

    let workspace = workspace(&cli);
    let output_name = cli.output_name();
    let gzip_out = cli.gzip_output();
    let short_name = cli.short_name();

    let extension = input_path
        .extension()
        .expect("failed to determine file extension")
        .to_string_lossy();

    let gzip_input = extension == "gz";

    let rinex = if gzip_input {
        Rinex::from_gzip_file(input_path)?
    } else {
        Rinex::from_file(input_path)?
    };

    let mut output = Output::new(
        &rinex,
        gzip_input,
        &workspace,
        gzip_out,
        short_name,
        output_name,
    );

    let mut rnx2bin = rinex
        .rnx2bin(meta)
        .unwrap_or_else(|| panic!("RNX2BIN internal failure"));

    binex_streaming(&mut output, &mut rnx2bin);
    Ok(())
}
