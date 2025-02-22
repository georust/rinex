use crate::datetime::{parse_datetime, ParseDateTimeError};
use thiserror::Error;

use crate::{header::is_valid_header, prelude::Epoch};

/// [BiasMode] defines how the Bias solutions to follow,
/// should be interpreted and used.
#[derive(Debug, PartialEq, Clone, Default)]
pub enum BiasMode {
    /// Relative Bias
    Relative,
    /// Absolute Bias
    #[default]
    Absolute,
}

#[derive(Debug, Error)]
pub enum BiasModeError {
    #[error("unknown bias mode")]
    UnknownBiasMode,
}

impl std::str::FromStr for BiasMode {
    type Err = BiasModeError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
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

#[derive(Debug, Clone)]
pub struct Header {
    /// SINEX Version for this file
    pub version: String,
    /// File creator agency code
    pub creator_code: String,
    /// Data provider agency code
    pub data_code: String,
    /// File creation [Epoch]
    pub creation_epoch: Epoch,
    /// Start time of solution
    pub start_epoch: Epoch,
    /// End time of solution
    pub end_epoch: Epoch,
    /// Relative or Absolute Bias mode
    pub bias_mode: BiasMode,
    /// Number of solutions in this file.
    pub num_solutions: u32,
}

impl std::str::FromStr for Header {
    type Err = Error;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        if !is_valid_header(content) {
            return Err(Error::MissingHeaderDelimiter);
        }
        if !content.starts_with("%=BIA") {
            return Err(Error::InvalidBiasHeader);
        }

        let (_, rem) = content.split_at(2); // marker
        let (_identifier, rem) = rem.split_at(4);
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
        assert!(header.is_ok());
        let header = header.unwrap();
        assert_eq!(header.version, "1.00");
        assert_eq!(header.creator_code, "PF2");
        assert_eq!(header.data_code, "PF2");
        assert_eq!(header.bias_mode, BiasMode::Relative);
        assert_eq!(header.length, 24);
        let content = "%=BIA 1.00 COD 2016:327:30548 IGS 2016:296:00000 2016:333:00000 A 00000194";
        let header = Header::from_str(content);
        assert!(header.is_ok());
        let header = header.unwrap();
        assert_eq!(header.version, "1.00");
        assert_eq!(header.creator_code, "COD");
        assert_eq!(header.bias_mode, BiasMode::Absolute);
    }
}
