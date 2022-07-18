use std::str::FromStr;
use thiserror::Error;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader};
use rinex::constellation::Constellation;

mod reference;
mod description;

pub mod bias;
pub mod header;
pub mod receiver;
pub mod datetime;
pub mod troposphere;

use header::Header;
use reference::Reference;

fn is_comment (line: &str) -> bool {
    line.starts_with("*")
}

fn section_start (line: &str) -> Option<String> {
    if line.starts_with("+") {
        Some(line[1..].to_string())
    } else {
        None
    }
}

fn section_end (line: &str) -> Option<String> {
    if line.starts_with("-") {
        Some(line[1..].to_string())
    } else {
        None
    }
}

fn is_dotdotdot (line: &str) -> bool {
    line.eq("...")
}

#[derive(Debug, Error)]
pub enum Error {
    /// SINEX file should start with proper header
    #[error("missing header delimiter")]
    MissingHeader,
    /// Failed to parse Header section
    #[error("invalid header content")]
    InvalidHeader,
    /// Closing incorrect section or structure is not correct
    #[error("faulty file structure")]
    FaultySection,
    /// Unknown section / category
    #[error("unknown type of section")]
    UnknownSection(String),
    /// Failed to open given file
    #[error("failed to open given file")]
    FileError(#[from] std::io::Error),
    /// Failed to parse integer number
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    /// Failed to parse Bias Mode
    #[error("failed to parse bias mode")]
    ParseBiasModeError(#[from] bias::BiasModeError),
    /// Failed to parse Determination Method
    #[error("failed to parse determination method")]
    ParseMethodError(#[from] bias::DeterminationMethodError),
    /// Failed to parse time system field
    #[error("failed to parse time system")]
    ParseTimeSystemError(#[from] bias::TimeSystemError),
}

/*
#[derive(Debug, Clone)]
pub enum Record {
    /// Bias (BIA) record case
    BiasSolutions(Vec<bias::Solution>),
    /// Troposphere (TRO) record case
    TropoRecord(troposphere::Record),
    // /// SINEX (SNX) record case
    //SinexRecord(sinex::Record)
}

impl Record {
    /// Unwraps Bias Solutions, if feasible
    pub fn bias_solutions (&self) -> Option<&Vec<bias::Solution>> {
        match self {
            Self::BiasSolutions(r) => Some(r),
            _ => None,
        }
    }
    /// Unwraps Troposphere Record, if feasible,
    /// is [troposphere::Record] definition for more detail
    pub fn tropo_record (&self) -> Option<&troposphere::Record> {
        match self {
            Self::TropoRecord(r) => Some(r),
            _ => None,
        }
    }
}
*/

#[derive(Debug, Clone)]
pub struct Sinex {
    /// Header section, is Document Type dependent
    pub header: Header,
    /// File Reference section
    pub reference: Reference,
    /// Possible `Input` Acknowledgemet, especially for data providers
    pub acknowledgments: Vec<String>,
    /// Possible `File Comments`
    pub comments: Vec<String>,
    /// File Description is Document Type dependent
    pub description: Description, 
    // /// Record
    //pub record: Record,
}

impl Sinex {
    pub fn from_file (file: &str) -> Result<Self, Error> {
        let file = std::fs::File::open(file)?;
        let reader = BufReader::new(file);
        let mut is_first = true;
        let mut bias_header = bias::header::Header::default();
        let mut tropo_header = troposphere::header::Header::default();
        let mut reference: Reference = Reference::default();
        let mut section = String::new();
        let mut comments : Vec<String> = Vec::new();
        let mut acknowledgments : Vec<String> = Vec::new();
        //let mut bias_description = bias::Description::default();
        //let mut bias_solutions: Vec<bias::Solution> = Vec::new();
        //let mut trop_description = troposphere::Description::default();
        //let mut trop_coordinates : Vec<troposphere::Coordinates> = Vec::new();
        for line in reader.lines() {
            let line = &line.unwrap();
            if is_comment(line) {
                continue
            }
            if is_dotdotdot(line) {
                continue
            }
            if is_first {
                if !is_header(line) {
                    return Err(Error::MissingHeader)
                }
                if let Ok(hd) = header::Header::from_str(line) {
                    header = hd.clone()
                }
                is_first = false;
                continue
            }

            if let Some(s) = section_start(line) {
                section = s.clone();
            
            } else if let Some(s) = section_end(line) {
                if !s.eq(&section) {
                    return Err(Error::FaultySection)
                }
            
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
                    "FILE/COMMENT" => {
                        comments.push(line.trim().to_string())
                    },
                    "INPUT/ACKNOWLEDGMENTS" => {
                        acknowledgments.push(line.trim().to_string())
                    },
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
                                let method = bias::DeterminationMethod::from_str(content.trim())?;
                                bias_description = bias_description.with_method(method)
                            },
                            "BIAS_MODE" => {
                                let mode = bias::BiasMode::from_str(content.trim())?;
                                bias_description = bias_description.with_bias_mode(mode)
                            },
                            "TIME_SYSTEM" => {
                                let system = bias::TimeSystem::from_str(content.trim())?;
                                bias_description = bias_description.with_time_system(system)
                            },
                            "RECEIVER_CLOCK_REFERENCE_GNSS" => {
                                if let Ok(c) = Constellation::from_1_letter_code(content.trim()) {
                                    bias_description = bias_description.with_rcvr_clock_ref(c)
                                }
                            },
                    "SATELLITE_CLOCK_REFERENCE_OBSERVABLES" => {
                                let items :Vec<&str> = content.trim()
                                    .split_ascii_whitespace()
                                    .collect();
                                if let Ok(c) = Constellation::from_1_letter_code(items[0]) {
                                    if items.len() == 1 {
                                        // --> no observable given
                                        let mut map: HashMap<Constellation, Vec<String>> = HashMap::new();
                                        map.insert(c, Vec::new());
                                        bias_description
                                            .sat_clock_ref = map.clone()
                                    } else {
                                        for i in 1..items.len() {
                                            bias_description = bias_description
                                                .with_sat_clock_ref(c, items[i])
                                        }
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    "TROP/DESCRIPTION" => {
                        let (descriptor, content) = line.split_at(41);
                        match descriptor.trim() {
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
                        }
                    },
                    "BIAS/SOLUTION" => {
                        if let Ok(sol) = bias::Solution::from_str(line.trim()) {
                            bias_solutions.push(sol)
                        }
                    },
                    "TROP/STA_COORDINATES" => {
                        if let Ok(coords) = troposphere::Coordinates::from_str(line.trim()) {
                            trop_coordinates.push(coords)
                        }
                    },
                    _ => return Err(Error::UnknownSection(section))
                }
            }
        }
        let doctype = header.doc_type.clone();
        Ok(Self {
            header: {
                match doctype {
                    header::DocumentType::BiasSolutions => header::BiasHeader(bias_header),
                    header::DocumentType::TropoCoordinates => header::TropoHeader(tropo_header),
                }
            },
            reference,
            acknowledgments,
            comments,
            description: {
                match doctype {
                    header::DocumentType::BiasSolutions => description::BiasDescription(bias_description),
                    header::DocumentType::TropoCoordinates => description::TropoDescription(trop_description),
                }
            },
            /*record: {
                match ftype {
                    header::DocumentType::BiasSolutions => Record::BiasSolutions(bias_solutions),
                    header::DocumentType::TropoCoordinates => Record::TropoCoordinates(trop_coordinates),
                }
            },*/
        })
    }
}
