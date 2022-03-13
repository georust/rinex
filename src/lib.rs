//! This package provides a set of tools to parse 
//! `RINEX` files.
//! 
//! Refer to README for example of use.  
//! Homepage: <https://github.com/gwbres/rinex>
mod meteo;
mod header;
mod version;
mod gnss_time;
mod navigation;
mod observation;

pub mod epoch;
pub mod record;
pub mod hatanaka;
pub mod constellation;

use thiserror::Error;
use std::str::FromStr;

#[macro_export]
/// Returns `true` if given `Rinex` line is a comment
macro_rules! is_comment {
    ($line: expr) => { $line.contains("COMMENT") };
}

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Type {
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
/// `Type` related errors
pub enum TypeError {
    #[error("Unknown RINEX type identifier \"{0}\"")]
    UnknownType(String),
}

impl Default for Type {
    /// Builds a default `Type`
    fn default() -> Type { Type::ObservationData }
}

impl Type {
    /// Converts `Self` to str
    pub fn to_str (&self) -> &str {
        match *self {
            Type::ObservationData => "ObservationData",
            Type::NavigationMessage => "NavigationMessage",
            Type::MeteorologicalData => "MeteorologicalData",
        }
    }
    /// Converts `Self` to string
    pub fn to_string (&self) -> String { String::from(self.to_str()) }
}

impl std::str::FromStr for Type {
    type Err = TypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") {
            Ok(Type::NavigationMessage)
        } else if s.contains("NAV DATA") {
            Ok(Type::NavigationMessage)
        } else if s.eq("OBSERVATION DATA") {
            Ok(Type::ObservationData)
        } else if s.eq("METEOROLOGICAL DATA") {
            Ok(Type::MeteorologicalData)
        } else {
            Err(TypeError::UnknownType(String::from(s)))
        }
    }
}

/// `Rinex` describes a `RINEX` file
#[derive(Debug)]
pub struct Rinex {
    /// `header` field contains general information
    pub header: header::RinexHeader,
    /// `record` contains `RINEX` file body
    /// and is type and constellation dependent 
    pub record: Option<record::Record>,
}

impl Default for Rinex {
    /// Builds a default `RINEX`
    fn default() -> Rinex {
        Rinex {
            header: header::RinexHeader::default(),
            record: None, 
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
    #[error("Rinex type error")]
    TypeError(#[from] TypeError),
}

impl Rinex {
    /// Builds a new `RINEX` struct from given:
    pub fn new (header: header::RinexHeader, record: Option<record::Record>) -> Rinex {
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

    /// Retruns true if this is an NAV rinex
    pub fn is_navigation_rinex (&self) -> bool { self.header.rinex_type == Type::NavigationMessage }
    /// Retruns true if this is an OBS rinex
    pub fn is_observation_rinex (&self) -> bool { self.header.rinex_type == Type::ObservationData }
    /// Returns true if this is a METEO rinex
    pub fn is_meteo_rinex (&self) -> bool { self.header.rinex_type == Type::MeteorologicalData }

    /// Builds a `Rinex` from given file.
    /// Input file must respect the whitespace specifications
    /// for the entire header section.   
    /// The header section must respect the labelization standard too.
    pub fn from_file (fp: &std::path::Path) -> Result<Rinex, RinexError> {
        let (header, body) = Rinex::split_rinex_content(fp)?;
        let header = header::RinexHeader::from_str(&header)?;
        let record : Option<record::Record> = match header.is_crinex() {
            false => Some(record::build_record(&header,&body)?),
            true => None,
        };
        Ok(Rinex { header, record })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    /// Tests `Rinex` constructor against all known test resources
    fn test_rinex_constructor() {
        let test_dir = env!("CARGO_MANIFEST_DIR").to_owned() + "/data";
        let types = vec![
			"NAV",
			"OBS",
			"MET"
		];
        for t in types {
            let versions = vec![
				"V2",
				"V3"
			];
            for v in versions {
                let dir_path = std::path::PathBuf::from(
                    test_dir.to_owned() + "/"+t + "/"+v
                );
                for entry in std::fs::read_dir(dir_path)
                    .unwrap() {
                    let entry = entry
                        .unwrap();
                    let path = entry.path();
					let is_hidden = entry.file_name()
						.to_str()
							.unwrap()
							.starts_with(".");
                    if !path.is_dir() && !is_hidden { // only relevant files..
                        let fp = std::path::Path::new(&path);
                        let rinex = Rinex::from_file(&fp);
                        assert_eq!(rinex.is_err(), false);
						let rinex = rinex.unwrap();
                        println!("File: {:?}\n{:#?}", &fp, rinex);
						match t {
							"NAV" => {
                                // NAV files sanity checks
                                assert_eq!(rinex.is_navigation_rinex(), true);
                            },
							"OBS" => {
                                // OBS files sanity checks
                                assert_eq!(rinex.is_observation_rinex(), true);
                                assert_eq!(rinex.header.obs_codes.is_some(), true);
                                if rinex.header.rcvr_clock_offset_applied {
                                    // epochs should always have a RCVR clock offset
                                    // test that with iterator
                                }
                            },
							"MET" => {
                                // METEO files sanity checks
                                assert_eq!(rinex.is_meteo_rinex(), true);
                                assert_eq!(rinex.header.met_codes.is_some(), true);
                            },
							_ => {},
						}
                    }
                }
            }
        }
    }
}
