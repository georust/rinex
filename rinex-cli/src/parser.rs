use rinex::epoch::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("duration format should be \"hh:mm:ss\"")]
    InvalidDurationFormat,
    #[error("non recognized epoch format")]
    NonRecognizedFormat,
}

/*
 * Parse Epoch From "Y-m-d H:M:S" description
 */
fn parse_epoch_ymd_hms(args: &str) -> Result<Epoch, Error> {
    let items: Vec<&str> = args
        .trim()
        .split(" ")
        .collect();
    if items.len() == 2 {

    }
}

/*
 * Parse Epoch from Y/M/D description
 */
fn parse_epoch_ymd(args: &str) -> Result<Epoch, Error> {
    let items: Vec<&str> = args
        .trim()
        .split("-")
        .collect();
    if items.len() >= 3 {
    
    }
}
    pub fn from_gregorian_utc(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, nanos: u32) -> Self {
    with_timescale()

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
                let total: f64 = (s + m * 60.0 + h*3600.0) as f64;
                Ok(Duration::from_seconds(total))
            } else {

            }
        } else {
        }
    } else {
    Err(Error::InvalidDurationFormat)
}

pub fn parse_epoch(args: &str) -> Result<Epoch, Error> {
    if Ok(e) = parse_epoch_ymd_hms(args) {
        Ok(e)
    } else if Ok(e) = parse_epoch_ymd(args) {
        Ok(e)
    } else {
        Err(Error::NonRecognizedFormat)
    }
}
