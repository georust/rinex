use std::str::FromStr;
use thiserror::Error;
use std::io::{prelude::*, BufReader};

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

pub enum DataType {
    ObsSampling,
    ParmeterSpacing,
    DeterminationMethod,
    BiasMode,
    TimeSystem,
    ReceiverClockRef,
    SatelliteClockReferenceObs,
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
}

pub struct Sinex {
    /// Header section
    pub header: header::Header,
    /// Possible `Input` Acknowledgemet, especially for data providers
    pub acknowledgments: Vec<String>,
    /// Possible `File Comments`
    pub comments: Vec<String>,
    /// Bias measurements / estimates,
    /// comprises a [Description] and a list of [Bias]
    pub bias: Vec<bias::Bias>,
}

impl Sinex {
    pub fn from_file (file: &str) -> Result<Self, Error> {
        let file = std::fs::File::open(file)?;
        let reader = BufReader::new(file);
        let mut is_first = true;
        let mut header = header::Header::default();
        let mut section = String::new();
        let mut comments : Vec<String> = Vec::new();
        let mut acknowledgments : Vec<String> = Vec::new();
        let mut bias: Vec<bias::Bias> = Vec::new();
        for line in reader.lines() {
            let line = &line.unwrap();
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

            if is_comment(line) {
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
                    },
                    "FILE/COMMENT" => {
                        comments.push(line.trim().to_string())
                    },
                    "INPUT/ACKNOWLEDGMENTS" => {
                        acknowledgments.push(line.trim().to_string())
                    },
                    "BIAS/DESCRIPTION" => {
                    },
                    "BIAS/SOLUTION" => {
                    },
                    _ => return Err(Error::UnknownSection(section))
                }
            }
        }
        Ok(Self {
            header,
            acknowledgments,
            comments,
            bias,
        })
    }
}
