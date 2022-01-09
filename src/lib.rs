//! This package provides a set of tools to parse 
//! `RINEX` files.
//! 
//! Refer to README for example of use.  
//! Homepage: <https://github.com/gwbres/rinex>
mod keys;
mod header;
mod meteo;
mod version;
mod gnss_time;
mod navigation;
mod observation;

pub mod record;
pub mod constellation;

use thiserror::Error;
use std::str::FromStr;

use header::RinexHeader;
use record::RinexRecord;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_rinex_comment {
    ($line: expr) => { $line.contains("COMMENT") };
}

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RinexType {
    /// Describes Observation Data (OBS),
    /// Phase & Pseudo range measurements
    ObservationData, 
    /// Describes Navigation Message (NAV)
    /// Ephemeride file
    NavigationMessage,
    /// Describes Meteorological data (Meteo)
    MeteorologicalData,
}

#[derive(Error, Debug)]
/// `RinexType` related errors
pub enum RinexTypeError {
    #[error("Unknown RINEX type identifier \"{0}\"")]
    UnknownType(String),
}

impl Default for RinexType {
    /// Builds a default `RinexType`
    fn default() -> RinexType { RinexType::ObservationData }
}

impl RinexType {
    /// Converts `Self` to str
    pub fn to_str (&self) -> &str {
        match *self {
            RinexType::ObservationData => "ObservationData",
            RinexType::NavigationMessage => "NavigationMessage",
            RinexType::MeteorologicalData => "MeteorologicalData",
        }
    }
    /// Converts `Self` to string
    pub fn to_string (&self) -> String { String::from(self.to_str()) }
}

impl std::str::FromStr for RinexType {
    type Err = RinexTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") {
            Ok(RinexType::NavigationMessage)
        } else if s.contains("NAV DATA") {
            Ok(RinexType::NavigationMessage)
        } else if s.eq("OBSERVATION DATA") {
            Ok(RinexType::ObservationData)
        } else if s.eq("METEOROLOGICAL DATA") {
            Ok(RinexType::MeteorologicalData)
        } else {
            Err(RinexTypeError::UnknownType(String::from(s)))
        }
    }
}

/// `Rinex` describes a `RINEX` file
#[derive(Debug)]
pub struct Rinex {
    header: RinexHeader,
    record: RinexRecord,
}

impl Default for Rinex {
    /// Builds a default `RINEX`
    fn default() -> Rinex {
        Rinex {
            header: RinexHeader::default(),
            record: Vec::new(),
        }
    }
}

#[derive(Error, Debug)]
/// `RINEX` Parsing related errors
pub enum RinexError {
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] header::Error),
}

impl Rinex {
    /// Builds a new `RINEX` struct from given:
    pub fn new (header: RinexHeader, record: RinexRecord) -> Rinex {
        Rinex {
            header,
            record,
        }
    }

    /// splits rinex file into two (header, body) contents
    fn split_rinex_content (fp: &std::path::Path) -> Result<(String, String), RinexError> {
        let content: String = std::fs::read_to_string(fp)
            .unwrap()
                .parse()
                .unwrap();
        let offset = match content.find(header::HEADER_END_MARKER) {
            Some(offset) => offset+13,
            _ => return Err(RinexError::MissingHeaderDelimiter)
        };
        let (header, body) = content.split_at(offset);
        Ok((String::from(header),String::from(body)))
    }

    /// Returns `Rinex` length, ie., number of record entries
    pub fn len (&self) -> usize { self.record.len() }
    /// Returns self's `header` section
    pub fn get_header (&self) -> &RinexHeader { &self.header }

    /// Returns entire RINEX record
    pub fn get_record (&self) -> &RinexRecord { &self.record }

    // Returns Record nth' entry
    //pub fn get_record_nth (&self, nth: usize) 
    //    -> &std::collections::HashMap<String, record::RecordItem> { &self.record[nth] }

    /// Retruns true if this is an NAV rinex
    pub fn is_navigation_rinex (&self) -> bool { self.header.get_rinex_type() == RinexType::NavigationMessage }
    /// Retruns true if this is an OBS rinex
    pub fn is_observation_rinex (&self) -> bool { self.header.get_rinex_type() == RinexType::ObservationData }
    /// Returns true if this is a METEO rinex
    pub fn is_meteo_rinex (&self) -> bool { self.header.get_rinex_type() == RinexType::MeteorologicalData }

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
        let header = RinexHeader::from_str(&header)?;

        // helpful information
        let rinex_type = header.get_rinex_type();
        let version = header.get_rinex_version();
        let version_major = version.get_major(); 
        let constellation = header.get_constellation();

        let mut body = body.lines();
        let mut line = body.next()
            .unwrap(); // ''END OF HEADER'' /BLANK

        while is_rinex_comment!(line) {
            line = body.next()
                .unwrap()
        }

        let mut eof = false;
        let mut first = true;
        let mut block = String::with_capacity(256*1024); // max. block size
        let mut record: RinexRecord = Vec::new();

        loop {
            let parsed: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            
            let is_new_block = record::block_record_start(&line, &rinex_type, &constellation, &version); 
            
            if is_new_block && !first {
                if let Ok(entry) = record::build_record_entry(&block, &header) {
                    record.push(entry)
                }
            }

            if is_new_block {
                if first {
                    first = false
                }
                block.clear()
            }

            block.push_str(&line);
            block.push_str("\n");

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
            record,
        })
    }
}

mod test {
    use super::*;
    #[test]
    /// Tests `Rinex` constructor against all known test resources
    fn test_rinex_constructor() {
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
