#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

/*
 * SINEX is part of the Geo-Rust framework.
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/georust/rinex/graphs/contributors)
 * This framework is shipped under both Apache-2.0 and MIT License.
 *
 * Documentation: https://github.com/georust/rinex
 */

mod description;
mod reference;

pub mod bias;
pub mod datetime;
pub mod error;
pub mod header;
pub mod receiver;
//pub mod troposphere;

extern crate gnss_rs as gnss;

use crate::{
    bias::{
        description::Description as BiasDescription, header::BiasMode, DeterminationMethod,
        Solution as BiasSolution, TimeSystem,
    },
    description::Description,
    error::ParsingError,
    header::{is_valid_header, Header},
    reference::Reference,
};

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
    str::FromStr,
};

use gnss::constellation::Constellation;
use hifitime::prelude::Epoch;

#[cfg(feature = "flate2")]
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};

pub type BiasSolutions = Vec<BiasSolution>;

fn is_comment(line: &str) -> bool {
    line.starts_with('*')
}

fn section_start(line: &str) -> Option<String> {
    if line.starts_with('+') {
        Some(line[1..].to_string())
    } else {
        None
    }
}

fn section_end(line: &str) -> Option<String> {
    if line.starts_with('-') {
        Some(line[1..].to_string())
    } else {
        None
    }
}

fn is_dotdotdot(line: &str) -> bool {
    line.eq("...")
}

/// Utility function to parse an [Epoch] from standardized SINEX format.
pub(crate) fn parse_epoch(content: &str) -> Result<Epoch, ParsingError> {
    if content.len() < 9 {
        ParsingError::EpochFormat;
    }

    let ym = &content[0..8]; // "YYYY:DDD"
    let dt = chrono::NaiveDate::parse_from_str(ym, "%Y:%j")?;
    let secs = &content[9..];
    let secs = f32::from_str(secs)?;
    let h = secs / 3600.0;
    let m = (secs - h * 3600.0) / 60.0;
    let s = secs - h * 3600.0 - m * 60.0;
    Ok(dt.and_hms(h as u32, m as u32, s as u32))
}

pub mod prelude {
    pub use gnss::prelude::Constellation;
    pub use hifitime::prelude::Epoch;
}

#[derive(Debug, Clone)]
pub enum Record {
    /// Bias (BIA) record case
    BiasSolutions(BiasSolutions),
    // /// Troposphere (TRO) record case
    // TropoRecord(troposphere::Record),
    // /// SINEX (SNX) record case
    //SinexRecord(sinex::Record)
}

impl Record {
    /// Unwraps Bias Solutions, if feasible
    pub fn bias_solutions(&self) -> Option<&BiasSolutions> {
        match self {
            Self::BiasSolutions(v) => Some(v),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sinex {
    /// [Header] is type dependent. Stores high level
    /// information.
    pub header: Header,
    /// File [Reference] section.
    pub reference: Reference,
    /// Possible `Input` Acknowledgemet, especially for data providers
    pub acknowledgments: Vec<String>,
    /// File [Description] is type dependent
    pub description: Description,
    /// [Record] stores the file content
    pub record: Record,
    /// Possible comments, stored as is.
    pub comments: Vec<String>,
}

impl Sinex {
    /// Parse [Sinex] from readable I/O.
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        let mut is_first = true;
        let mut header = Header::default();
        let mut reference: Reference = Reference::default();
        let mut section = String::new();
        let mut comments: Vec<String> = Vec::new();
        let mut acknowledgments: Vec<String> = Vec::new();
        let mut bias_description = BiasDescription::default();
        let mut bias_solutions = BiasSolutions::with_capacity(16);

        //let mut trop_description = troposphere::Description::default();
        //let mut trop_coordinates : Vec<troposphere::Coordinates> = Vec::new();
        for line in reader.lines() {
            let line = &line.unwrap();
            if is_comment(line) {
                continue;
            }
            if is_dotdotdot(line) {
                continue;
            }
            if is_first {
                if !is_valid_header(line) {
                    return Err(ParsingError::MissingHeader);
                }
                if let Ok(hd) = Header::from_str(line) {
                    header = hd.clone()
                }
                is_first = false;
                continue;
            }

            if let Some(s) = section_start(line) {
                section = s.clone();
            } else if let Some(s) = section_end(line) {
                if !s.eq(&section) {
                    return Err(ParsingError::FaultySection);
                }
            } else if is_valid_header(line) {
                break; // EOF
            } else {
                match section.as_str() {
                    "FILE/REFERENCE" => {
                        let (descriptor, content) = line.split_at(19);
                        match descriptor.trim() {
                            "INPUT" => reference = reference.with_input(content.trim()),
                            "OUTPUT" => reference = reference.with_output(content.trim()),
                            "DESCRIPTION" => reference = reference.with_description(content.trim()),
                            "SOFTWARE" => reference = reference.with_software(content.trim()),
                            "HARDWARE" => reference = reference.with_hardware(content.trim()),
                            "CONTACT" => reference = reference.with_contact(content.trim()),
                            _ => {},
                        }
                    },
                    "FILE/COMMENT" => comments.push(line.trim().to_string()),
                    "INPUT/ACKNOWLEDGMENTS" => acknowledgments.push(line.trim().to_string()),
                    "BIAS/DESCRIPTION" => {
                        let (descriptor, content) = line.split_at(41);
                        match descriptor.trim() {
                            "OBSERVATION_SAMPLING" => {
                                let sampling = u32::from_str_radix(content.trim(), 10)?;
                                bias_description = bias_description.with_sampling(sampling)
                            },
                            "PARAMETER_SPACING" => {
                                let spacing = u32::from_str_radix(content.trim(), 10)?;
                                bias_description = bias_description.with_spacing(spacing)
                            },
                            "DETERMINATION_METHOD" => {
                                let method = DeterminationMethod::from_str(content.trim())?;
                                bias_description = bias_description.with_method(method)
                            },
                            "BIAS_MODE" => {
                                let mode = BiasMode::from_str(content.trim())?;
                                bias_description = bias_description.with_bias_mode(mode)
                            },
                            "TIME_SYSTEM" => {
                                let system = TimeSystem::from_str(content.trim())?;
                                bias_description = bias_description.with_time_system(system)
                            },
                            "RECEIVER_CLOCK_REFERENCE_GNSS" => {
                                if let Ok(c) = Constellation::from_str(content.trim()) {
                                    bias_description = bias_description.with_rcvr_clock_ref(c)
                                }
                            },
                            "SATELLITE_CLOCK_REFERENCE_OBSERVABLES" => {
                                let items: Vec<&str> =
                                    content.trim().split_ascii_whitespace().collect();
                                if let Ok(c) = Constellation::from_str(items[0]) {
                                    if items.len() == 1 {
                                        // --> no observable given
                                        let mut map: HashMap<Constellation, Vec<String>> =
                                            HashMap::new();
                                        map.insert(c, Vec::new());
                                        bias_description.sat_clock_ref = map.clone()
                                    } else {
                                        for i in 1..items.len() {
                                            bias_description =
                                                bias_description.with_sat_clock_ref(c, items[i])
                                        }
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    /*
                    "TROP/DESCRIPTION" => {
                        let (descriptor, content) = line.split_at(41);
                        /*match descriptor.trim() {
                            "ELEVATION CUTOFF ANGLE" => {
                                let angle = u32::from_str_radix(content.trim(), 10)?;
                            },
                            "SAMPLING INTERVAL" => {
                                let interval = u32::from_str_radix(content.trim(), 10)?;
                                trop_description = trop_description
                                    .with_sampling_interval(interval)
                            },
                            "SAMPLING TROP" => {
                                let interval = u32::from_str_radix(content.trim(), 10)?;
                                trop_description = trop_description
                                    .with_tropo_sampling(interval)
                            },
                            "TROP MAPPING FUNCTION" => {
                                let functions :Vec<&str> = content
                                    .split_ascii_whitespace()
                                    .collect();
                                for func in functions {
                                    trop_description = trop_description
                                        .with_mapping_function(func)
                                }
                            },
                            "SOLUTION_FIELDS_1" => {
                                let fields :Vec<&str> = content
                                    .split_ascii_whitespace()
                                    .collect();
                                for field in fields {
                                    trop_description = trop_description
                                        .with_solution_field(field)
                                }
                            },
                            _ => {},
                        }*/
                    },*/
                    "BIAS/SOLUTION" => {
                        if let Ok(sol) = bias::Solution::from_str(line.trim()) {
                            bias_solutions.push(sol)
                        }
                    },
                    /*
                    "TROP/STA_COORDINATES" => {
                        if let Ok(coords) = troposphere::Coordinates::from_str(line.trim()) {
                            trop_coordinates.push(coords)
                        }
                    },*/
                    _ => return Err(ParsingError::UnknownSection(section)),
                }
            }
        }
        //let doctype = header.doc_type.clone();
        Ok(Self {
            header,
            reference,
            acknowledgments,
            comments,
            description: Description::BiasDescription(bias_description),
            record: Record::BiasSolutions(bias_solutions),
        })
    }

    /// Parse [Sinex] from local file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let path = path.as_ref();

        let file = File::open(path).unwrap_or_else(|e| panic!("Failed to open file: {}", e));

        let mut reader = BufReader::new(file);
        Self::parse(&mut reader)
    }

    /// Parse [Sinex] from local gzip compressed file
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<Self, ParsingError> {
        let path = path.as_ref();

        let file = File::open(path).unwrap_or_else(|e| panic!("Failed to open file: {}", e));

        let reader = GzDecoder::new(fd);
        let mut reader = BufReader::new(reader);
        Self::parse(&mut reader)
    }
}

#[cfg(test)]
mod test {
    use crate::{parse_epoch, ParsingError};

    #[test]
    fn epoch_parsing() {
        match parse_epoch("2022:02") {
            Ok(_) => panic!("epoch parsing should have failed!"),
            Err(e) => match e {
                ParsingError::EpochFormat => {},
                e => panic!("parser returned invalid error: {}", e),
            },
        }

        let _ = parse_epoch("2022:021:20823").unwrap();

        let _ = parse_epoch("2022:009:00000").unwrap();
    }
}
