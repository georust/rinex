//! `Epoch` description
use thiserror::Error;
use std::str::FromStr;

/// An `Epoch` is an observation timestamp
pub type Epoch = chrono::NaiveDateTime;

#[derive(Error, Debug)]
pub enum ParseEpochError {
    #[error("failed to parse seconds")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse y:m:d-h:m")]
    ParseIntError(#[from] std::num::ParseIntError),
}

pub fn from_string (s: &str) -> Result<Epoch, ParseEpochError> {
    let items : Vec<&str> = s.split_ascii_whitespace().collect();
    let (mut y,m,d,h,min,s) : (i32,u32,u32,u32,u32,f64) =
        (i32::from_str_radix(items[0],10)?,
         u32::from_str_radix(items[1],10)?,
         u32::from_str_radix(items[2],10)?,
         u32::from_str_radix(items[3],10)?,
         u32::from_str_radix(items[4],10)?,
         f64::from_str(items[5])?);
    if y < 100 {
        y += 2000
    }
    Ok(chrono::NaiveDate::from_ymd(y,m,d)
        .and_hms(h,min,s as u32))
}
