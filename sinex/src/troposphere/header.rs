use thiserror::Error;
use crate::bias;
use crate::header::is_valid_header;
use crate::datetime::{parse_datetime, ParseDateTimeError};

/// List of known Techniques to generate
/// the Tropospheric solutions
#[derive(Debug, Clone)]
pub enum Technique {
    /// A combination of techniques was used
    Combined,
    /// DORIS
    DORIS,
    /// GNSS
    GNSS,
    /// VLBI
    VLBI,
    /// Water Vapour,
    WaterVapour,
    /// Radio sounding
    RadioSounding,
    /// (Numerical) Weather forecast
    WeatherForecast,
    /// (Numerical) Weather re-analysis
    WeatherReanalysis,
    /// Climate model
    ClimateModel,
}

/// Technique Parsing Error
#[derive(Debug, Clone)]
pub enum TechniqueParsingError {
    /// Unknown Technique descriptor
    UnknownTechnique(String),
}

impl std::str::FromStr for Technique {
    type Err = TechniqueError;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("P") {
            Ok(Self::GNSS)
        } else if content.eq("C") {
            Ok(Self::Combined)
        } else if content.eq("D") {
            Ok(Self::DORIS)
        } else if content.eq("R") {
            Ok(Self::VLBI) 
        } else if content.eq("W") {
            Ok(Self::WaterVapour)
        } else if content.eq("S") {
            Ok(Self::RadioSounding)
        } else if content.eq("F") {
            Ok(Self::WeatherForecast)
        } else if content.eq("N") {
            Ok(Self::WeatherReanalysis)
        } else if content.eq("M") {
            Ok(Self::ClimateModel)
        } else {
            Err(TechniqueError::UnknownTechnique(content.to_string()))
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    /// Header line should start with %=BIA
    #[error("missing header delimiter")]
    MissingHeaderDelimiter,
    #[error("not a troposphere header")]
    NonTropoHeader,
    /// Non recognized file type
    #[error("file type error")]
    FileTypeError(#[from] FileTypeError),
    #[error("failed to parse datetime")]
    ParseDateTimeError(#[from] ParseDateTimeError),
    #[error("failed to parse `length` field")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse `bias_mode` field")]
    BiasModeError(#[from] bias::BiasModeError),
}

#[derive(Debug, Clone)]
pub struct Header {
    /// Revision for this file
    pub version: String,
    /// File creator agency code
    pub creator_code: String,
    /// Data provider agency code
    pub provider_code: String,
    /// File creation date
    pub date: chrono::NaiveDateTime,
    /// Start time of solution
    pub start_time: chrono::NaiveDateTime,
    /// End time of solution
    pub end_time: chrono::NaiveDateTime,
    /// Relative or Absolute Bias mode
    pub bias_mode: bias::BiasMode,
    /// Number of bias estimates in this file
    pub length: u32,
}

impl std::str::FromStr for Header {
    type Err = Error;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if !is_valid_header(content) {
            return Err(Error::MissingHeaderDelimiter)
        }
        if !content.starts_with("=%TRO") {
            return Err(Error::NonTropoHeader)
        }
        let (_, rem) = content.split_at(2); // marker
        let (identifier, rem) = rem.split_at(4);
        let (version, rem) = rem.split_at(5);
        let (file_code, rem) = rem.split_at(4);
        let (creation, rem) = rem.split_at(15);
        let (provider_code, rem) = rem.split_at(4);
        let (start_time, rem) = rem.split_at(15);
        let (end_time, rem) = rem.split_at(15);
        let (bias_mode, rem) = rem.split_at(2);
        let length = u32::from_str_radix(rem.trim(), 10)?;
        Ok(Self {
            version: version.trim().to_string(),
            creator_code: file_code.trim().to_string(),
            date: parse_datetime(creation.trim())?,
            provider_code: provider_code.trim().to_string(),
            start_time: parse_datetime(start_time.trim())?,
            end_time: parse_datetime(end_time.trim())?,
            length,
            bias_mode: bias::BiasMode::from_str(bias_mode.trim())?,
        
        })
    }
}

impl Default for Header {
    fn default() -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            version: String::from("1.00"),
            creator_code: String::from("Unknown"),
            provider_code: String::from("Unknown"),
            length: 0,
            date: now,
            start_time: now,
            end_time: now,
            bias_mode: bias::BiasMode::default(),
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
    }
}
