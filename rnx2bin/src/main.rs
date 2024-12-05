mod cli;
use cli::Cli;

use rinex::prelude::{binex::RNX2BIN, FormattingError, ParsingError, Rinex};

use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use flate2::{write::GzEncoder, Compression};
use thiserror::Error;

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

fn gzip_binex_streaming<P: AsRef<Path>>(path: P, rnx2bin: &mut RNX2BIN) {
    let fd = File::create(path).unwrap_or_else(|e| panic!("File.creation error: {}", e));
    let mut fd = GzEncoder::new(fd, Compression::new(5));

    let mut buf = [0; 2048];

    while let Some(msg) = rnx2bin.next() {
        msg.encode(&mut buf, 2048)
            .unwrap_or_else(|e| panic!("BINEX encoding error: {:?}", e));

        buf = [0; 2048];

        fd.write(&buf)
            .unwrap_or_else(|e| panic!("File.write error: {}", e));
    }
}

fn binex_streaming<P: AsRef<Path>>(path: P, rnx2bin: &mut RNX2BIN) {
    let mut fd = File::create(path).unwrap_or_else(|e| panic!("File.creation error: {}", e));

    let mut buf = [0; 2048];

    while let Some(msg) = rnx2bin.next() {
        msg.encode(&mut buf, 2048)
            .unwrap_or_else(|e| panic!("BINEX encoding error: {:?}", e));

        buf = [0; 2048];

        fd.write(&buf)
            .unwrap_or_else(|e| panic!("File.write error: {}", e));
    }
}

fn main() -> Result<(), Error> {
    let cli = Cli::new();

    let meta = cli.meta();
    let input_path = cli.input_path();
    let output_name = cli.output_name();
    let workspace = workspace(&cli);
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

    let mut rnx2bin = rinex.rnx2bin(meta).expect("RNX2BIN internal failure!");

    // output path
    let output_path = if let Some(output_name) = output_name {
        workspace.join(output_name)
    } else {
        let mut suffix = ".bin".to_string();

        if gzip_input || gzip_out {
            suffix.push_str(".gz");
        }

        let auto = rinex.standard_filename(short_name, Some(&suffix), None);
        workspace.join(auto)
    };

    let output_file_name = output_path
        .file_name()
        .expect("failed to determine output file")
        .to_string_lossy();

    println!("generating {}", output_file_name);

    if gzip_input || gzip_out {
        gzip_binex_streaming(output_path, &mut rnx2bin);
    } else {
        binex_streaming(output_path, &mut rnx2bin);
    }

    Ok(())
}
