use chrono::{
    Duration,
    NaiveDate,
    NaiveDateTime,
};
use thiserror::Error;
use time::OutOfRangeError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("format should be %HH:%MM:%SS to describe a duration")]
    InvalidDurationFormat,
    #[error("duration parsing overflowed")]
    DurationOverflow(#[from] OutOfRangeError),
}

pub fn parse_duration(args: &str) -> Result<Duration, Error> {
    let hms:Vec<&str> = args
        .split(":")
        .collect();
    if hms.len() != 3 {
        return Err(Error::InvalidDurationFormat)
    }
    
    if let Ok(h) = u64::from_str_radix(hms[0], 10) {
        if let Ok(m) = u64::from_str_radix(hms[1], 10) {
            if let Ok(s) = u64::from_str_radix(hms[2], 10) {
                let std = std::time::Duration::from_secs(
                    h*3600 + m*60 + s);
                return Ok(Duration::from_std(std)?)
            }
        }
    }
    Err(Error::InvalidDurationFormat)
}

pub fn parse_date (args: &str) -> Result<NaiveDate, chrono::format::ParseError> {
    chrono::NaiveDate
        ::parse_from_str(args, "%Y-%m-%d")
}

pub fn parse_datetime (args: &str) -> Result<NaiveDateTime, chrono::format::ParseError> {
    chrono::NaiveDateTime
        ::parse_from_str(args, "%Y-%m-%d %H:%M:%S")
}

