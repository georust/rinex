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
mod version;
mod constellation;

#[macro_export]
macro_rules! is_rinex_comment  {
    ($line: expr) => { $line.contains("COMMENT") };
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

        if !header.get_rinex_version().is_supported() {
            //TODO
        }

        let mut body = body.lines();
        let mut line = body.next()
            .unwrap(); // ''END OF HEADER'' /BLANK

        while is_rinex_comment!(line) {
            line = body.next()
                .unwrap()
        }

        let rtype = header.get_rinex_type();
        let version = header.get_rinex_version();
        let version_major = version.get_major(); 
        let version_minor = version.get_minor(); 
        let constellation = header.get_constellation();
        
        let mut record = String::with_capacity(256*1024);
        let (mut record_start, mut record_end) = (false, false);
        let mut eof = false;
        let mut first = true;

        let nav_message_to_match: &'static [char]  = &
            ['R','G','E','B','J','C','S']; 

        loop {
            //let parsed: Vec<&str> = line.split_ascii_whitespace()
            //    .collect();

            /* builds record grouping */
            match rtype {
                header::RinexType::NavigationMessage => {
                    match constellation {
                        constellation::Constellation::Glonass => {},
                        _ => {
                            record_start = nav_message_to_match.contains(&line.chars().nth(0)
                                .unwrap())
                        },
                    }
                },
                header::RinexType::ObservationData => {
                    // uses nb of float numbers
                    if version_major < 3 {
                        if version_minor < 11 {
                        } else {
                        }
                    } else {
                        // marker based
                    }
                },
                _ => {}
            }

            if record_start {
                if first {
                    first = false
                } else {
                    //TODO
                    //process previous block
                    println!("RECORD: \"{}\"", record)
                }

                record_start = false;
                record.clear()
            }
                
            record.push_str(&line);

            if let Some(l) = body.next() {
                line = l
            } else {
                break
            }

            while is_rinex_comment!(line) {
                if let Some(l) = body.next() {
                    line = l
                } else {
                    eof = true; 
                    break 
                }
            }

            if eof {
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
