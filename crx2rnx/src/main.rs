//! Command line tool to decompress CRINEX files 
use rinex::{header, hatanaka};
use rinex::hatanaka::Decompressor;
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
    #[error("decompression error")]
    DecompressionError(#[from] hatanaka::Error),
}

fn main() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml)
        .setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();
    let filepath = matches.value_of("filepath")
        .unwrap();

    let max_compression_order = matches.value_of("max-compression-order").unwrap_or("6");
    let max_compression_order = u16::from_str_radix(max_compression_order, 10).unwrap();

    let mut outpath = String::from("output.crx");
    if let Some(prefix) = filepath.strip_suffix("d") { // CRNX1
        outpath = prefix.to_owned() + "o" // RNX1
    } else {
        if let Some(prefix) = filepath.strip_suffix(".crx") { // CRNX3
            outpath = prefix.to_owned() + "rnx" // RNX3
        }
    }

    let outpath : String = String::from(matches.value_of("output")
        .unwrap_or(&outpath));
    let output = std::fs::File::create(outpath.clone())?;
    decompress(filepath, max_compression_order.into(), output)?;
    println!("{} generated", outpath);
    Ok(())
}

/// Decompresses given file,   
/// fp : filepath   
/// m : maximal compression order for core algorithm    
/// writer: stream
fn decompress (fp: &str, m: usize, mut writer: std::fs::File) -> Result<(), Error> {
    // BufferedReader is not efficient enough (at the moment) to
    // perform the Hatanaka decompression by itself, but we'll get there..
    // BufferedReader supports .gzip stream decompression and efficient .line() browsing.
    let reader = BufferedReader::new(fp)?;
    let mut inside_crinex = true;
    for line in reader.lines() {
        let l = &line.unwrap();
        if !inside_crinex {
            // push header fields as is
            write!(writer, "{}\n", l)?
        }
        if l.contains("CRINEX PROG / DATE") {
            inside_crinex = false
        } else if l.contains("END OF HEADER") {
            break
        }
    }
    
    // parse header fields
    // we need them to determine things when decompressing the record
    let mut reader = BufferedReader::new(fp)?;
    let header = header::Header::new(&mut reader)?;
    // decompress file body
    let mut decompressor = Decompressor::new(m)
        .unwrap();
    for l in reader.lines() {
        let line = &l.unwrap();
        if line.len() == 0 {
            // sometimes we run into empty lines
            // like omitted clock offset fields,
            // and decompress() does not like it
            let recovered = decompressor.decompress(&header, " ")?;
            write!(writer, "{}", recovered)?
        } else {
            let recovered = decompressor.decompress(&header, line)?;
            write!(writer, "{}", recovered)?
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use assert_cmd::prelude::*;
    use std::process::Command;
    /// Runs `diff` to determine whether f1 & f2 
    /// are strictly identical or not. Whitespaces are omitted
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
    /// The test bench consists in calling `crx2rnx` as is,
    /// on each /CRNX/Vx test resource where
    /// we do have an /OBS/Vy counterpart.
    /// We uncompress the file and perform a `file diff` which
    /// must returns 0. The hidden trick: /CRNX/Vx and its counterpart
    /// comprise the same epoch content
    #[test]
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
                let counterpart_name = obs_counterpart.file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                println!("decompressing \"{}\" against \"{}\"", file_stem, counterpart_name);
                let mut cmd = Command::cargo_bin("crx2rnx")?;
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
    /// The test bench consists in calling `crx2rnx` as is,
    /// on each /CRNX/Vx test resource where
    /// we do have an /OBS/Vy counterpart.
    /// We uncompress the file and perform a `file diff` which
    /// must returns 0. The hidden trick: /CRNX/Vx and its counterpart
    /// comprise the same epoch content
    #[test]
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
                let counterpart_name = obs_counterpart.file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap();
                println!("decompressing \"{}\" against \"{}\"", file_stem, counterpart_name);
                let mut cmd = Command::cargo_bin("crx2rnx")?;
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
