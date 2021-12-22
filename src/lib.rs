//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! This lib is work in progress
//! 
//! Homepage: <https://github.com/gwbres/rinex>

use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

mod header;
mod constellation;

use header::RinexType;

#[macro_export]
macro_rules! is_rinex_comment  {
    ($line: expr) => { $line.contains("COMMENT") };
}

macro_rules! version_major {
    ($version: expr) => {
        u8::from_str_radix(
            $version.split_at(
                $version.find(".")
                    .unwrap())
                        .0, 10).unwrap()
    };
}

macro_rules! version_minor {
    ($version: expr) => {
        u8::from_str_radix(
            $version.split_at(
                $version.find(".")
                    .unwrap()+1)
                        .1, 10).unwrap()
    };
}

/// `Rinex` main structure,
/// describes a `RINEX` file
#[derive(Debug)]
struct Rinex {
    header: header::Header,
//    body: Vec<T>, // body frames
}

#[derive(Error, Debug)]
enum RinexError {
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] header::HeaderError),
}

impl Rinex {
    /// Builds a Rinex struct
    pub fn new (header: header::Header) -> Rinex {
        Rinex {
            header
        }
    }

    /// splits rinex file into two
    /// (header, body) as strings
    fn split_rinex_content (fp: &std::path::Path) -> Result<(String, String), RinexError> {
        let content: String = std::fs::read_to_string(fp)
            .unwrap()
                .parse()
                .unwrap();
        let offset = match content.find(header::HEADER_END_MARKER) {
            Some(offset) => offset,
            _ => return Err(RinexError::MissingHeaderDelimiter)
        };
        let (header, body) = content.split_at(offset);
        Ok((String::from(header),String::from(body)))
        //let parsed: Vec<&str> = content.split(header::HEADER_END_MARKER)
        //    .collect();
        //Ok(parsed[0].to_string())
    }

    /// Builds a `Rinex` from given file.
    /// Input file must respect the whitespace specifications
    /// for the entire header section.   
    /// The header section must respect the labelization standard too.
    pub fn from_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let name = fp.file_name()
            .unwrap();
        let extension = fp.extension()
            .unwrap();
        let extension = extension.to_str()
            .unwrap();

        let (header, body) = Rinex::split_rinex_content(fp)?;
        let header = header::Header::from_str(&header)?;
        let mut body = body.lines();
        let mut line = body.next()
            .unwrap(); // ''END OF HEADER'' /BLANK

        while is_rinex_comment!(line) {
            line = body.next()
                .unwrap()
        }

        let mut next_line = body.next()
            .unwrap();

        while is_rinex_comment!(next_line) {
            next_line = body.next()
                .unwrap()
        }

        let version = header.get_rinex_version();
        let version_major = version_major!(version);
        let version_minor = version_minor!(version);
        
        let mut record = String::with_capacity(256*1024);
        let (mut record_start, mut record_end) = (true, false);
        let mut eof = false;

        loop {
            
            record.push_str(&line);

            let parsed: Vec<&str> = line.split_ascii_whitespace()
                .collect();

            let parsed_next: Vec<&str> = next_line.split_ascii_whitespace()
                .collect();

            /* builds record grouping */
            match header.get_rinex_type() {
                RinexType::ObservationData => {
                    if version_major < 3 {
                        if version_minor < 11 {
                            // 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26
                            // -14746974.73049 -11440396.20948  22513484.6374   22513484.7724   22513487.3704
                            record_start = parsed.len() == 9;
                            record_end = parsed_next.len() == 5 
                        } else {

                            
                        }

                    } 
                },
                _ => {}
            }

            if record_end {
                record_end = false;
                record_start = true;
                println!("RECORD: \"{}\"", record);
                break
            }

            if record_start {
                record.clear(); 
                record_start = false
            }

            if let Some(l) = body.next() {
                line = next_line;
                next_line = l
            } else {
                break
            }

            while is_rinex_comment!(next_line) {
                if let Some(l) = body.next() {
                    next_line = l
                } else {
                    eof = true; 
                    break 
                }
            }

            if (eof) {
                break
            }
        }

        Ok(Rinex{
            header, 
        })
    }
}

mod test {
    use super::*;
/*
    /// tests Rcvr object fromStr method
    fn rcvr_from_str() {
        assert_eq!(
            Rcvr::from_str("82205               LEICA RS500         4.20/1.39  ")
                .unwrap(),
            Rcvr{
                sn: String::from("82205"),
                model: String::from("LEICA RS500"),
                firmware: String::from("4.20/1.39")
            });
    }
*/
    #[test]
    /// Test `Rinex` constructor
    /// against all valid data resources
    fn rinex_constructor() {
        // open test resources
        let test_resources = std::path::PathBuf::from(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/data");
        // walk test resources
        for entry in std::fs::read_dir(test_resources)
            .unwrap() {
            let entry = entry
                .unwrap();
            let path = entry.path();
            if !path.is_dir() { // only files..
                let fp = std::path::Path::new(&path);
                let rinex = Rinex::from_file(&fp);
                assert_eq!(rinex.is_err(), false);
                println!("File: {:?}\n{:#?}", &fp, rinex)
            }
        }
    }
}
