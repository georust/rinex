use thiserror::Error;
use crate::bias;
use crate::header;
use crate::header::is_valid_header;
use crate::datetime::{parse_datetime, ParseDateTimeError};

#[derive(Debug, PartialEq, Clone)]
/// Describes how the included GNSS
/// bias values have to be interpreted and applied
pub enum BiasMode {
    /// Relative Bias 
    Relative,
    /// Abslute Bias 
    Absolute,
}

#[derive(Debug, Error)]
pub enum BiasModeError {
    #[error("unknown bias mode")]
    UnknownBiasMode,
}

impl Default for BiasMode {
    fn default() -> Self {
        Self::Absolute
    }
}

impl std::str::FromStr for BiasMode {
    type Err = BiasModeError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("R") {
            Ok(BiasMode::Relative)
        } else if content.eq("RELATIVE") {
            Ok(BiasMode::Relative)
        } else if content.eq("A") {
            Ok(BiasMode::Absolute)
        } else if content.eq("ABSOLUTE") {
            Ok(BiasMode::Absolute)
        } else {
            Err(BiasModeError::UnknownBiasMode)
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// Header line should start with %=
    #[error("missing header delimiter")]
    MissingHeaderDelimiter,
    /// Header line should start with %=BIA
    #[error("not a bias header")]
    NonBiasHeader,
    /// Non recognized file type
    #[error("file type error")]
    FileTypeError(#[from] header::DocumentTypeError),
    #[error("failed to parse datetime")]
    ParseDateTimeError(#[from] ParseDateTimeError),
    #[error("failed to parse `length` field")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse `bias_mode` field")]
    BiasModeError(#[from] BiasModeError),
}

#[derive(Debug, Clone)]
pub struct Header {
    /// SINEX Version for this file
    pub version: String,
    /// File creator agency code
    pub creator_code: String,
    /// Data provider agency code
    pub data_code: String,
    /// File creation date
    pub date: chrono::NaiveDateTime,
    /// Start time of solution
    pub start_time: chrono::NaiveDateTime,
    /// End time of solution
    pub end_time: chrono::NaiveDateTime,
    /// Relative or Absolute Bias mode
    pub bias_mode: BiasMode,
    /// Number of bias estimates in this file
    pub length: u32,
}

impl std::str::FromStr for Header {
    type Err = Error;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if !is_valid_header(content) {
            return Err(Error::MissingHeaderDelimiter)
        }
        if !content.starts_with("%=BIA") {
            return Err(Error::NonBiasHeader)
        }

        let (_, rem) = content.split_at(2); // marker
        let (identifier, rem) = rem.split_at(4);
        let (version, rem) = rem.split_at(5);
        let (file_code, rem) = rem.split_at(4);
        let (creation, rem) = rem.split_at(15);
        let (data_code, rem) = rem.split_at(4);
        let (start_time, rem) = rem.split_at(15);
        let (end_time, rem) = rem.split_at(15);
        let (bias_mode, rem) = rem.split_at(2);
        let length = u32::from_str_radix(rem.trim(), 10)?;
        Ok(Self {
            version: version.trim().to_string(),
            creator_code: file_code.trim().to_string(),
            date: parse_datetime(creation.trim())?,
            data_code: data_code.trim().to_string(),
            start_time: parse_datetime(start_time.trim())?,
            end_time: parse_datetime(end_time.trim())?,
            length,
            bias_mode: BiasMode::from_str(bias_mode.trim())?,
        })
    }
}

impl Default for Header {
    fn default() -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            version: String::from("1.00"),
            creator_code: String::from("Unknown"),
            data_code: String::from("Unknown"),
            length: 0,
            date: now,
            start_time: now,
            end_time: now,
            bias_mode: BiasMode::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_header() {
        let content = "%=BIA 1.00 PF2 2011:180:59736 PF2 2011:113:86385 2011:114:86385 R 00000024";
        let header = Header::from_str(content);
        assert_eq!(header.is_ok(), true);
        let header = header.unwrap();
        assert_eq!(header.version, "1.00");
        assert_eq!(header.creator_code, "PF2");
        assert_eq!(header.data_code, "PF2");
        assert_eq!(header.bias_mode, BiasMode::Relative);
        assert_eq!(header.length, 24);
    }
}
