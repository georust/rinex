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

impl Output {
    pub fn new(
        rinex: &Rinex,
        gzip_in: bool,
        workspace: &Path,
        gzip_out: bool,
        short_name: bool,
        io_output: bool,
        custom_name: Option<&String>,
    ) -> Self {
        if let Some(custom) = custom_name {
            // custom output specified
            if io_output {
                // this must be a fullpath: workspace is disregarded
                let mut fd = File::open(custom)
                    .unwrap_or_else(|e| panic!("Failed to create file within workspace"));

                if gzip_out {
                    println!("Gzip streaming to I/O interface: {}", custom);
                    let mut fd = GzEncoder::new(fd, Compression::new(5));
                    Output::GzipIo(fd)
                } else {
                    println!("Streaming to I/O interface: {}", custom);
                    Output::Io(fd)
                }
            } else {
                let path = workspace.join(custom);

                let mut fd = File::create(&path)
                    .unwrap_or_else(|e| panic!("Failed to create file within workspace"));

                if gzip_in || gzip_out {
                    println!("Generating custom gzip file: {}", path);
                    let mut fd = GzEncoder::new(fd, Compression::new(5));
                    Output::GzipFile(fd)
                } else {
                    println!("Generating custom file: {}", path);
                    Output::File(fd)
                }
            }
        } else {
            // auto generated name
            let mut suffix = ".bin".to_string();

            if gzip_in || gzip_out {
                suffix.push_str(".gz");
            }

            let auto = rinex.standard_filename(short_name, Some(suffix), None);

            let path = workspace.join(auto);

            let mut fd = File::create(&path)
                .unwrap_or_else(|e| panic!("Failed to create file within workspace"));

            if gzip_in || gzip_out {
                println!("Generating gzip file: {}", path);
                let mut fd = GzEncoder::new(fd, Compression::new(5));
                Output::GzipFile(fd)
            } else {
                println!("Generating file: {}", path);
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

//fn binex_streaming<W: Write>(output: Output<W>, rnx2bin: &mut RNX2BIN) {
//    const BUF_SIZE: usize = 4096;
//    let mut buf = [0; BUF_SIZE];
//
//    while let Some(msg) = rnx2bin.next() {
//        msg.encode(&mut buf, BUF_SIZE)
//            .unwrap_or_else(|e| panic!("BINEX encoding error: {:?}", e));
//
//        buf = [0; BUF_SIZE];
//
//        output
//            .write(&buf)
//            .unwrap_or_else(|e| panic!("I/O error: {}", e));
//    }
//}

fn main() -> Result<(), Error> {
    let cli = Cli::new();

    Ok(())
}
