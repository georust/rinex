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
    
    let date = matches.value_of("date");
    let time = matches.value_of("time");

    // output filepath
    let mut output_path = String::from("output.crx");
    if let Some(stripped) = filepath.strip_suffix("o") { // RNX1 
        output_path = stripped.to_owned() + "d" // CRNX1
    } else {
        if let Some(stripped) = filepath.strip_suffix("rnx") { // RNX3
            output_path = stripped.to_owned() + "crx" // CRNX3
        }
    }

    //output filepath
    let output_path = String::from(matches.value_of("output").unwrap_or(&output_path));
    let output = std::fs::File::create(output_path.clone())?;
    //RNX2CRX compression
    compress(filepath, output, date, time)?;
    println!("{} generated", output_path);
    Ok(())
}

fn compress (
        fp: &str, 
        mut writer: std::fs::File, 
        date: Option<&str>, 
        time: Option<&str>,
    ) -> Result<(), Error> { 
    
    // compression date
    let date = match date {
        Some(date) => {
            match time {
                Some(time) => {
                    let descriptor = date.to_owned() + " " + time;
                    chrono::NaiveDateTime::parse_from_str("%d-%m-%Y %H:%M", &descriptor)
                        .unwrap()
                },
                _ => chrono::NaiveDateTime::parse_from_str("%d-%m-%Y", date)
                    .unwrap(),
            }
        },
        None => {
            match time {
                Some(time) => {
                    let time = chrono::NaiveTime::parse_from_str("%H-%M-%S", time)
                        .unwrap();
                    chrono::Utc::now().date()
                        .and_time(time)
                        .unwrap()
                        .naive_utc()
                },
                _ => chrono::Utc::now().naive_utc(),
            }
        },
    };

    // write CRINEX infos 
    let crinex = Crinex {
        version: Version {
            major: 1,
            minor: 0,
        },
        date,
        prog: "rust-rnx2crx".to_string(),
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
        if let Some(_) = &obs.crinex {
            panic!("this file is already compressed");
        }
    }

    // BufferedWriter is not efficient enough (at the moment) to
    // perform the Hatanaka compression by itself, but we'll get there..
    let mut compressor = Compressor::new();
    // compress file body
    for l in reader.lines() {
        let line = &l.unwrap();
        let compressed = compressor.compress(&header, line)?;
        write!(writer, "{}", compressed)?
    }
    Ok(())
}
