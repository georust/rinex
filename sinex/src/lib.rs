use std::str::FromStr;
use thiserror::Error;
use std::io::{prelude::*, BufReader};
use rinex::constellation::Constellation;

pub mod bias;
pub mod header;
pub mod receiver;

fn is_comment (line: &str) -> bool {
    line.starts_with("*")
}

fn is_header (line: &str) -> bool {
    line.starts_with("%=BIA")
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

fn is_end_bias (line: &str) -> bool {
    line.starts_with("%=ENDBIA")
}

fn is_dotdotdot (line: &str) -> bool {
    line.eq("...")
}

#[derive(Debug, Error)]
pub enum ParseDateTimeError {
    #[error("failed to parse YYYY:DDD")]
    DatetimeError(#[from] chrono::format::ParseError),
    #[error("failed to parse SSSSS")]
    ParseSecondsError(#[from] std::num::ParseFloatError),
}

fn parse_datetime (content: &str) -> Result<chrono::NaiveDateTime, ParseDateTimeError> {
    let ym = &content[0..8]; // "YYYY:DDD"
    let dt = chrono::NaiveDate::parse_from_str(&ym, "%Y:%j")?;
    let secs = &content[9..];
    let secs = f32::from_str(secs)?;
    let h = secs /3600.0;
    let m = (secs - h*3600.0)/60.0;
    let s = secs - h*3600.0 - m*60.0;
    Ok(dt.and_hms(h as u32, m as u32, s as u32))
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

#[derive(Debug, Clone)]
pub struct Reference {
    /// Organization(s) providing / gathering file content
    pub description: String,
    /// Brief description of the input used to generate the solution
    pub input: String,
    /// Description of the file contents
    pub output: String,
    /// Address of the relevant contact (email..)
    pub contact: String,
    /// Software used to generate this file
    pub software: String,
    /// Hardware used to genreate this file
    pub hardware: String,
}

impl Reference {
    pub fn with_description (&self, description: &str) -> Self {
        Self {
            description: description.to_string(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_input (&self, input: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: input.to_string(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_output (&self, output: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: output.to_string(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_contact (&self, contact: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: contact.to_string(),
            software: self.software.clone(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_software (&self, software: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: software.to_string(),
            hardware: self.hardware.clone(),
        }
    }
    pub fn with_hardware (&self, hardware: &str) -> Self {
        Self {
            description: self.description.clone(),
            input: self.input.clone(),
            output: self.output.clone(),
            contact: self.contact.clone(),
            software: self.software.clone(),
            hardware: hardware.to_string(),
        }
    }
}

impl Default for Reference {
    fn default() -> Self {
        Self {
            description: String::from("?"),
            input: String::from("?"),
            output: String::from("?"),
            contact: String::from("unknown"),
            software: String::from("unknown"),
            hardware: String::from("unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sinex {
    /// Header section
    pub header: header::Header,
    /// File Reference section
    pub reference: Reference,
    /// Possible `Input` Acknowledgemet, especially for data providers
    pub acknowledgments: Vec<String>,
    /// Possible `File Comments`
    pub comments: Vec<String>,
    /// Bias Description
    pub description: bias::Description,
    /// Bias estimate solutions 
    pub solutions: Vec<bias::Solution>,
}

impl Sinex {
    pub fn from_file (file: &str) -> Result<Self, Error> {
        let file = std::fs::File::open(file)?;
        let reader = BufReader::new(file);
        let mut is_first = true;
        let mut header = header::Header::default();
        let mut reference: Reference = Reference::default();
        let mut section = String::new();
        let mut comments : Vec<String> = Vec::new();
        let mut acknowledgments : Vec<String> = Vec::new();
        let mut description = bias::Description::default();
        let mut solutions: Vec<bias::Solution> = Vec::new();
        for line in reader.lines() {
            let line = &line.unwrap();
            if is_comment(line) {
                continue
            }
            if is_end_bias(line) {
                break
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
                        println!("DESCRIPTION \"{}\"", descriptor);
                        match descriptor.trim() {
                            "OBSERVATION_SAMPLING" => {
                                let sampling = u32::from_str_radix(content.trim(), 10)?;
                                description = description.with_sampling(sampling)
                            },
                            "PARAMETER_SPACING" => {
                                let spacing = u32::from_str_radix(content.trim(), 10)?;
                                description = description.with_spacing(spacing)
                            },
                            "DETERMINATION_METHOD" => {
                                let method = bias::DeterminationMethod::from_str(content.trim())?;
                                description = description.with_method(method)
                            },
                            "BIAS_MODE" => {
                                let mode = bias::BiasMode::from_str(content.trim())?;
                                description = description.with_bias_mode(mode)
                            },
                            "TIME_SYSTEM" => {
                                let system = bias::TimeSystem::from_str(content.trim())?;
                                description = description.with_time_system(system)
                            },
                            "RECEIVER_CLOCK_REFERENCE_GNSS" => {
                                if let Ok(c) = Constellation::from_1_letter_code(content.trim()) {
                                    description = description.with_rcvr_clock_ref(c)
                                }
                            },
                    "SATELLITE_CLOCK_REFERENCE_OBSERVABLES" => {
                                let items :Vec<&str> = content.trim()
                                    .split_ascii_whitespace()
                                    .collect();
                                if let Ok(c) = Constellation::from_1_letter_code(items[0]) {
                                    for i in 1..items.len() {
                                        description = description
                                            .with_sat_clock_ref(c, items[i])
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    "BIAS/SOLUTION" => {
                    },
                    _ => return Err(Error::UnknownSection(section))
                }
            }
        }
        Ok(Self {
            header,
            reference,
            acknowledgments,
            comments,
            description,
            solutions,
        })
    }
}
