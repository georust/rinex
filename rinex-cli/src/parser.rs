use rinex::epoch::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("duration format should be \"hh:mm:ss\"")]
    DurationError,
    #[error("date format should be \"yyyy-mm-dd\"")]
    DateError,
    #[error("datetime format should be \"yyyy-mm-dd hh:mm:ss\"")]
    DatetimeError,
    #[error("non recognized epoch format")]
    FormatError,
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
        let ymd: Vec<&str> = items[0].split("-").collect();
        let hms: Vec<&str> = items[1].split(":").collect();
        if let Ok(y) = i32::from_str_radix(ymd[0], 10) {
            if let Ok(m) = u8::from_str_radix(ymd[1], 10) {
                if let Ok(d) = u8::from_str_radix(ymd[2], 10) {
                    if let Ok(hh) = u8::from_str_radix(hms[0], 10) {
                        if let Ok(mm) = u8::from_str_radix(hms[1], 10) {
                            if let Ok(ss) = u8::from_str_radix(hms[2], 10) {
                                return Ok(Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0));
                            }
                        }
                    }
                }
            }
        }
    } 
    Err(Error::DatetimeError)
}

/*
 * Parse Epoch From "Y-m-d" description
 */
fn parse_epoch_ymd(args: &str) -> Result<Epoch, Error> {
    let items: Vec<&str> = args
        .trim()
        .split("-")
        .collect();
    if items.len() == 3 {
        if let Ok(h) = i32::from_str_radix(items[0].trim(), 10) {
            if let Ok(m) = u8::from_str_radix(items[1].trim(), 10) {
                if let Ok(s) = u8::from_str_radix(items[2].trim(), 10) {
                    return Ok(Epoch::from_gregorian_utc_midnight(h, m, s));
                }
            }
        }
    }
    Err(Error::DateError)
}

pub fn parse_duration(args: &str) -> Result<Duration, Error> {
    let hms:Vec<&str> = args
        .split(":")
        .collect();
    if hms.len() == 3 {
        if let Ok(h) = u64::from_str_radix(hms[0], 10) {
            if let Ok(m) = u64::from_str_radix(hms[1], 10) {
                if let Ok(s) = u64::from_str_radix(hms[2], 10) {
                    let total: f64 = (s + m * 60 + h*3600) as f64;
                    return Ok(Duration::from_seconds(total));
                }
            }
        }
    }
    Err(Error::DurationError)
}

pub fn parse_epoch(args: &str) -> Result<Epoch, Error> {
    if let Ok(e) = parse_epoch_ymd_hms(args) {
        Ok(e)
    } else if let Ok(e) = parse_epoch_ymd(args) {
        Ok(e)
    } else {
        Err(Error::FormatError)
    }
}
