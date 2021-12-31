//! This package provides a set of tools to parse 
//! and analyze RINEX files.
//! 
//! Homepage: <https://github.com/gwbres/rinex>

mod header;
mod version;
mod gnss_time;

pub mod record;
pub mod constellation;

use thiserror::Error;
use std::str::FromStr;
use scan_fmt::scan_fmt;

use record::*;
use version::RinexVersion;
use constellation::Constellation;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_rinex_comment {
    ($line: expr) => { $line.contains("COMMENT") };
}

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RinexType {
    ObservationData,
    NavigationMessage,
    MeteorologicalData,
    ClockData,
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
    /// Converts `Self` to string
    pub fn to_string (&self) -> &str {
        match *self {
            RinexType::ObservationData => "ObservationData",
            RinexType::NavigationMessage => "NavigationMessage",
            RinexType::MeteorologicalData => "MeteorologicalData",
            RinexType::ClockData => "ClockData",
        }
    }
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
        } else {
            Err(RinexTypeError::UnknownType(String::from(s)))
        }
    }
}

/// `Rinex` main structure,
/// describes a `RINEX` file
#[derive(Debug)]
pub struct Rinex {
    header: header::RinexHeader,
    records: Vec<record::RinexRecord>,
}

impl Default for Rinex {
    /// Builds a default `RINEX`
    fn default() -> Rinex {
        Rinex {
            header: header::RinexHeader::default(),
            records: Vec::new(),
        }
    }
}

#[derive(Error, Debug)]
pub enum RinexError {
    #[error("Header delimiter not found")]
    MissingHeaderDelimiter,
    #[error("Header parsing error")]
    HeaderError(#[from] header::Error),
}

/// macro to return true when a new block record
/// has been identified
pub fn new_record_block (line: &str,
    rinex_type: &RinexType,
        constellation: &Constellation, 
            version: &RinexVersion) -> bool
{
    let major = version.get_major();
    let parsed: Vec<&str> = line.split_ascii_whitespace()
        .collect();
    
    match major < 4 {
        true => {
            // RinexType:: dependent
            match rinex_type {
                RinexType::NavigationMessage => {
                    let known_sv_identifiers: &'static [char] = 
                        &['R','G','E','B','J','C','S']; 
                    match constellation {
                        Constellation::Glonass => parsed.len() > 4,
                        _ => {
                            match line.chars().nth(0) {
                                Some(c) => known_sv_identifiers.contains(&c), 
                                _ => false
                                    //TODO
                                    // <o 
                                    //   for some files we end up with "\n xxxx" as first frame items 
                                    // current code will discard first payload item in such scenario
                                    // => need to cleanup (split(head,body) method)
                            }
                        }
                    }
                },
                RinexType::ObservationData => parsed.len() > 8,
                _ => false, 
            }
        },
        false => {      
            // V4: OBS blocks have a '>' delimiter
            match line.chars().nth(0) {
                Some(c) => c == '>',
                _ => false,
                    //TODO
                    // <o 
                    //   for some files we end up with "\n xxxx" as first frame items 
                    // current code will discard first payload item in such scenario
                    // => need to cleanup (split(head,body) method)
            }
        },
    }
}

impl Rinex {
    /// Builds a Rinex struct
    pub fn new (header: header::RinexHeader, records: Vec<record::RinexRecord>) -> Rinex {
        Rinex {
            header,
            records,
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

    /// Returns `Rinex` length, ie.,
    ///   nb of observations for `RinexType::ObservationData`   
    ///   nb of ephemerides  for `RinexType::NavigationMessage`   
    pub fn len (&self) -> usize { self.records.len() }

    /// Returns self's `header` section
    pub fn get_header (&self) -> &header::RinexHeader { &self.header }

    /// Returns entire Rinex Record
    pub fn get_records (&self) -> &Vec<record::RinexRecord> { &self.records }

    /// Returns nth record identified in self
    pub fn get_record (&self, nth: usize) -> &record::RinexRecord { &self.records[nth] }

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
        let header = header::RinexHeader::from_str(&header)?;

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
        let mut records: Vec<RinexRecord> = Vec::new();

        loop {
            let parsed: Vec<&str> = line.split_ascii_whitespace()
                .collect();
            
            let new_block = new_record_block(&line, &rinex_type, &constellation, &version); 

            // Build new record
            if new_block && !first {
                let record: Option<RinexRecord> = match rinex_type {
                    RinexType::NavigationMessage => {
                        if let Ok(record) = 
                            navigation::NavigationRecord::from_string(version, constellation, &block) {
                                Some(RinexRecord::RinexNavRecord(record))
                        } else {
                            None
                        }
                    },
                    RinexType::ObservationData => {
                        if let Ok(record) =
                            observation::ObservationRecord::from_string(version, constellation, &block) {
                                Some(RinexRecord::RinexObsRecord(record))
                        } else {
                            None
                        }
                    },
                    _ => None,
                };
                
                if record.is_some() {
                    records.push(record.unwrap())
                }
            }

            if new_block {
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
            records,
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
