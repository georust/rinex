//! Command line tool to compress RINEX data
use rinex::version::Version;
use rinex::{header, hatanaka};
use rinex::hatanaka::Compressor;
use rinex::observation::Crinex;
use rinex::reader::BufferedReader;

use clap::{
    App, AppSettings,
    load_yaml,
};

use thiserror::Error;
use std::io::{Write, BufRead};

#[derive(Error, Debug)]
enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse RINEX header")]
    ParseHeaderError(#[from] header::Error),
    #[error("compression error")]
    CompressionError(#[from] hatanaka::Error),
}

fn main() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();
    
    // input filepath
    let filepath = matches
        .value_of("filepath")
        .unwrap();
    
    // output filepath
    let mut output_path = String::from("output.crx");
    if let Some(stripped) = filepath.strip_suffix("o") { // RNX1 
        output_path = stripped.to_owned() + "d" // CRNX1
    } else {
        if let Some(stripped) = filepath.strip_suffix("rnx") { // RNX3
            output_path = stripped.to_owned() + "crx" // CRNX3
        }
    }

    //RNX2CRX compression
    let output = std::fs::File::create(output_path.clone())?;
    compress(filepath, output)?;
    println!("{} generated", output_path);
    Ok(())
}

fn compress (fp: &str, mut writer: std::fs::File) -> Result<(), Error> {
    // write CRINEX infos 
    let crinex = Crinex {
        version: Version {
            major: 1,
            minor: 0,
        },
        prog: "rust-rnx2crx".to_string(),
        date: chrono::Utc::now().naive_utc(),
    };
    write!(writer, "{}", crinex)?;

    // BufferedReader supports .gzip stream decompression 
    // and efficient .line() browsing.
    let reader = BufferedReader::new(fp)?;

    for line in reader.lines() {
        let l = &line.unwrap();
        // push header fields as is
        write!(writer, "{}\n", l)?;
        if l.contains("END OF HEADER") {
            break 
        }
    }

    // build Header structure from header section only
    // this mainly serves for observable identification
    let mut reader = BufferedReader::new(fp)?;
    let header = header::Header::new(&mut reader)?;
    if let Some(obs) = &header.obs {
        if let Some(crinex) = &obs.crinex {
            panic!("this file is already compressed");
        }
    }

    // BufferedWriter is not efficient enough (at the moment) to
    // perform the Hatanaka compression by itself, but we'll get there..
    let mut compressor = Compressor::new(5)
        .unwrap();
    // compress file body
    for l in reader.lines() {
        let line = &l.unwrap();
        let compressed = compressor.compress(&header, line)?;
        write!(writer, "{}", compressed)?
    }
    Ok(())
}
