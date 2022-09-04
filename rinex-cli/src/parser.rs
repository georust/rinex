use thiserror::Error;

/// Duration parsing (from user input) related issues
#[derive(Debug, Error)]
pub enum DurationError {
    /// We expect %HH:%MM:%SS format
    #[error("format should be %HH:%MM:%SS")]
    InvalidFormat,
    /// Specified duration / parsed duration induced an overflow
    #[error("time internal overflow!")]
    TimeOutOfRange(#[from] time::OutOfRangeError),
}

/// Parses an std::time::Duration from user input
pub fn parse_duration (content: &str) 
        -> Result<chrono::Duration, DurationError> 
{
    let hms : Vec<_> = content.split(":").collect();
    if hms.len() == 3 {
        if let Ok(h) =  u64::from_str_radix(hms[0], 10) {
            if let Ok(m) =  u64::from_str_radix(hms[1], 10) {
                if let Ok(s) =  u64::from_str_radix(hms[2], 10) {
                    let std = std::time::Duration::from_secs(h*3600 + m*60 +s);
                    return Ok(chrono::Duration::from_std(std)?)
                }
            }
        }
    }
    Err(DurationError::InvalidFormat)
}

/// Parses an chrono::NaiveDateTime from user input
pub fn parse_datetime (content: &str) 
        -> Result<chrono::NaiveDateTime, chrono::format::ParseError> 
{
    chrono::NaiveDateTime::parse_from_str(content, "%Y-%m-%d %H:%M:%S")
}

/// `Epoch` parsing related issues
#[derive(Error, Debug)]
pub enum EpochError {
    #[error("chrono format error")]
    ChronoFormatError(#[from] chrono::format::ParseError),
    #[error("std::io error")]
    IoError(#[from] std::io::Error),
}

use rinex::epoch;
use std::str::FromStr;
    
/// Parses an `epoch` from user input
pub fn parse_epoch (content: &str) -> Result<epoch::Epoch, EpochError> {
    let format = "YYYY-MM-DD HH:MM:SS";
    if content.len() > format.len() { // an epoch flag was given
        Ok(epoch::Epoch {
            date: parse_datetime(&content[0..format.len()])?,
            flag: epoch::EpochFlag::from_str(&content[format.len()..].trim())?,
        })
    } else { // no epoch flag given
        // --> we associate an Ok flag
        Ok(epoch::Epoch {
            date: parse_datetime(content)?,
            flag: epoch::EpochFlag::Ok,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_duration_parser() {
        let duration = parser::parse_duration("00:30:00");
        assert_eq!(duration.is_ok(), true);
        let duration = duration.unwrap();
        assert_eq!(duration, chrono::Duration::minutes(30));
        let duration = parser::parse_duration("30:00");
        assert_eq!(duration.is_err(), true);
        let duration = parser::parse_duration("00 30 00");
        assert_eq!(duration.is_err(), true);
    }
    #[test]
    fn test_epoch_parser() {
        let epoch = parse_epoch("2022-03-01 00:30:00");
        assert_eq!(epoch.is_ok(), true);
        let epoch = epoch.unwrap();
        assert_eq!(epoch, epoch::Epoch {
            date: parse_datetime("2022-03-01 00:30:00").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
    }
}
