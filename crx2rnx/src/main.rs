//! crx2rnx: 
//! command line tool to compress RINEX files   
//! and decompress CRINEX files
use rinex::{header, hatanaka};

use clap::App;
use clap::load_yaml;
use thiserror::Error;
use std::io::{Write, BufRead, BufReader};

#[derive(Error, Debug)]
enum Error {
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("failed to parse RINEX header")]
    ParseHeaderError(#[from] header::Error),
    #[error("hatanaka error")]
    HatanakaError(#[from] hatanaka::Error),
}

fn main() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
    let matches = app.get_matches();
    let filepath = matches.value_of("filepath")
        .unwrap();
    let m = u16::from_str_radix(matches.value_of("m")
        .unwrap_or("8"),10).unwrap();
    let _strict_flag = matches.is_present("strict");

    let mut default_output = String::from("output.crx");
    if filepath.ends_with("d") { // CRNX < 3
        default_output = filepath.strip_suffix("d")
            .unwrap()
            .to_owned() + "o";
    } else if filepath.ends_with(".crx") { // CRNX >= 3
        default_output = filepath.strip_suffix(".crx")
            .unwrap()
            .to_owned() + ".rnx";
    }

    let outpath : String = String::from(matches.value_of("output")
        .unwrap_or(&default_output));
    let output = std::fs::File::create(outpath.clone())?;
    decompress(filepath, m, output)?;
    println!("{} generated", outpath);
    Ok(())
}

/// Decompresses given file,   
/// fp : filepath   
/// m : maximal compression order for core algorithm    
/// writer: stream
fn decompress (fp: &str, m: u16, mut writer: std::fs::File) -> Result<(), Error> {
    let input = std::fs::File::open(fp)?;
    let reader = BufReader::new(input);
    let header = header::Header::new(fp)?;
    let mut end_of_header = false;
    let mut decompressor = hatanaka::Decompressor::new(m.into());
    println!("Decompressing file \"{}\"", fp);
    for l in reader.lines() {
        let line = &l.unwrap();
        if !end_of_header {
            if !line.contains("CRINEX VERS") && !line.contains("CRINEX PROG") {
                // strip CRINEX special header
                writeln!(writer, "{}", line)?
            }
            if line.contains("END OF HEADER") {
                // identify header section
                println!("RINEX Header parsed");
                // reset for record section
                end_of_header = true;
            }
        } else { // RINEX record
            let mut content = line.to_string();
            if content.len() == 0 {
                content = String::from(" ");
            }
            let recovered = decompressor.decompress(&header, &content)?;
            write!(writer, "{}", recovered)?
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use assert_cmd::prelude::*;
    use std::process::Command;
    /// Runs `diff` to determines whether f1 & f2 
    /// are strictly identical or not
    fn diff_is_strictly_identical (f1: &str, f2: &str) -> Result<bool, std::string::FromUtf8Error> {
        let output = Command::new("diff")
            .arg("-q")
            .arg("-Z")
            .arg(f1)
            .arg(f2)
            .output()
            .expect("failed to execute \"diff\"");
        let output = String::from_utf8(output.stdout)?;
        Ok(output.len()==0)
    }
    #[test]
    /// Tests CRINEX1 decompression
    fn test_decompression_v1()  -> Result<(), Box<dyn std::error::Error>> { 
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CRNX/V1";
        let comparison_resources = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/OBS/V2";
        let path = std::path::PathBuf::from(test_resources.to_owned());
        for e in std::fs::read_dir(path).unwrap() {
            let entry = e.unwrap();
            let path = entry.path();
            // make sure this is a CRINEX file
            let file_stem = path.file_stem()
                .unwrap()
                    .to_str()
                    .unwrap();
            let file_extension = path.extension() 
                .unwrap()
                    .to_str()
                    .unwrap();
            let is_hidden = file_stem.starts_with("."); 
            let is_crinex = !path.is_dir() && !is_hidden && file_extension.ends_with("d");
            // make sure we do have a mirror RINEX file, to run testbench
            let obs_counterpart = std::path::PathBuf::from(
                comparison_resources.to_owned() + 
                    "/" +file_stem + "."  // file with same name
                    +&file_extension[..file_extension.len()-1] +"o" // but OBS extension
            );
            if is_crinex && obs_counterpart.exists() {
                println!("decompressing file \"{}\"", file_stem);
                let mut cmd = Command::cargo_bin("hatanaka")?;
                cmd.arg("-f")
                   .arg(&path)
                   .arg("-o")
                   .arg("testv1.rnx");
                cmd.assert()
                   .success();
                // compare produced OBS and mirror OBS using `diff`
                let identical = diff_is_strictly_identical(
                    "testv1.rnx", 
                    obs_counterpart.to_str().unwrap()) // mirror OBS file
                        .unwrap(); 
                assert_eq!(identical,true)
            }
        }
        // remove generated file
        let _ = std::fs::remove_file("testv1.rnx");
        Ok(())
    }
    #[test]
    /// Tests CRINEX3 decompression
    fn test_decompression_v3()  -> Result<(), Box<dyn std::error::Error>> {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CRNX/V3";
        let comparison_resources = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/OBS/V3";
        let path = std::path::PathBuf::from(test_resources.to_owned());
        for e in std::fs::read_dir(path).unwrap() {
            let entry = e.unwrap();
            let path = entry.path();
            // make sure this is a CRINEX file
            let file_stem = path.file_stem()
                .unwrap()
                    .to_str()
                    .unwrap();
            let file_extension = path.extension() 
                .unwrap()
                    .to_str()
                    .unwrap();
            let is_hidden = file_stem.starts_with("."); 
            let is_crinex = !path.is_dir() && !is_hidden && file_extension == "crx";
            // make sure we do have a mirror RINEX file, to run testbench
            let obs_counterpart = std::path::PathBuf::from(
                comparison_resources.to_owned() + 
                    "/" +file_stem // file with same name
                    +".rnx" // but OBS extension
            );
            if is_crinex && obs_counterpart.exists() {
                println!("decompressing file \"{}\"", file_stem);
                let mut cmd = Command::cargo_bin("hatanaka")?;
                cmd.arg("-f")
                   .arg(&path)
                   .arg("-o")
                   .arg("testv3.rnx");
                cmd.assert()
                   .success();
                // compare produced OBS and mirror OBS using `diff`
                let identical = diff_is_strictly_identical(
                    "testv3.rnx", 
                    obs_counterpart.to_str().unwrap()) // mirror OBS file
                        .unwrap(); 
                assert_eq!(identical,true)
            }
        }
        // remove generated file
        let _ = std::fs::remove_file("testv3.rnx");
        Ok(())
    }
}
