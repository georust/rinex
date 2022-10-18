//! `Epoch` is an observation timestamp with
//! a `flag` associated to it
use thiserror::Error;
use std::str::FromStr;
use chrono::{Datelike,Timelike};

mod flag;
pub use flag::EpochFlag;

#[cfg(feature = "serde")]
use serde::{Serialize};

/// An `Epoch` is an observation timestamp 
/// with an [epoch::EpochFlag] associated to it
#[derive(Copy, Clone, Debug)]
#[derive(PartialOrd, Ord)]
#[derive(PartialEq, Eq, Hash)]
pub struct Epoch {
    /// Sampling timestamp
    pub date: chrono::NaiveDateTime,
    /// Epoch flag
    pub flag: flag::EpochFlag,
}

#[cfg(feature = "serde")]
impl Serialize for Epoch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{} {}", 
            self.date.format("%Y-%m-%d %H:%M:%S"),
            self.flag.to_string());
        serializer.serialize_str(&s)
    }
}

impl Default for Epoch {
    fn default() -> Epoch {
        let now = chrono::Utc::now();
        Epoch {
            flag: EpochFlag::default(),
            date: chrono::NaiveDate::from_ymd(
                now.naive_utc().date().year(),
                now.naive_utc().date().month(),
                now.naive_utc().date().day())
                    .and_hms(
                        now.time().hour(),
                        now.time().minute(),
                        now.time().second())
        }
    }
}

impl Epoch {
    /// Builds a new `Epoch` from given flag & timestamp
    pub fn new (date: chrono::NaiveDateTime, flag: EpochFlag) -> Epoch {
        Epoch { 
            date,
            flag,
        }
    }
	/// Builds an `epoch` to describe current instant
	pub fn now() -> Self {
		Self::default()
	}
	/// Builds an `epoch` with desired customized flag
	pub fn with_flag(&self, flag: EpochFlag) -> Self {
		Self {
			date: self.date,
			flag,
		}
	}
    /// Formats self in OBS/V2 compatible format
    pub fn to_string_obs_v2(&self) -> String {
        let date = self.date.format("%y %_m %_d %_H %_M %_S.%6f");
        format!("{}0  {}", date, self.flag) // adds 1 extra 0, we would like %7f to match standards
    }
    /// Formats self in NAV/V2 compatible format
    pub fn to_string_nav_v2(&self) -> String {
        self.date.format("%y %m %d %H %M %S.0")
            .to_string()
    }
    /// Formats self in OBS/V3 compatible format
    pub fn to_string_obs_v3(&self) -> String {
        let date = self.date.format("%Y %m %d %H %M %S.%6f");
        format!("{}  {}", date, self.flag)
    }
    /// Formats self in NAV/V3 compatible format
    pub fn to_string_nav_v3(&self) -> String {
        self.date.format("%Y %m %d %H %M %S")
            .to_string()
    }
}

#[derive(Error, Debug)]
/// `epoch.date` field parsing related errors
pub enum ParseDateError {
    #[error("format mismatch, expecting yy mm dd hh mm ss.ssss")]
    FormatMismatch, 
    #[error("failed to parse seconds field")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse y/m/d h:m fields")]
    ParseIntError(#[from] std::num::ParseIntError),
}

/// Builds an `epoch.date` field from "yyyy mm dd hh mm ss.sssss"
/// content, as generally found in `RINEX` epoch descriptors
pub fn str2date (s: &str) 
        -> Result<chrono::NaiveDateTime, ParseDateError> 
{
    let items : Vec<&str> = s.split_ascii_whitespace().collect();
    if items.len() != 6 {
        return Err(ParseDateError::FormatMismatch)
    }
    let mut secs: u32 = 0;
    let (mut y,m,d,h,min) : (i32,u32,u32,u32,u32) =
        (i32::from_str_radix(items[0],10)?,
         u32::from_str_radix(items[1],10)?,
         u32::from_str_radix(items[2],10)?,
         u32::from_str_radix(items[3],10)?,
         u32::from_str_radix(items[4],10)?);
    if let Ok(s) = f64::from_str(items[5].trim()) {
        secs = s as u32
    }
    else if let Ok(s) = u32::from_str_radix(items[5].trim(), 10) {
        secs = s 
    }
	if y < 100 { // 2 digit nb case
    	if y > 90 { // old rinex
        	y += 1900
    	} else {
			y += 2000
		}
	}
    Ok(chrono::NaiveDate::from_ymd(y,m,d)
        .and_hms(h,min,secs))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_str2date() {
        assert_eq!(str2date("22 01 01 00 00 00").is_ok(), true);
        assert_eq!(str2date("22 01 01 00 00").is_ok(), false);
        let date = str2date("2022 01 01 00 00 00");
        assert_eq!(date.is_ok(), true);
        let date = date.unwrap();
        assert_eq!(date.date().year(), 2022);
        assert_eq!(date.date().month(), 01);
        assert_eq!(date.date().day(), 01);
        assert_eq!(date.time().hour(), 0);
        assert_eq!(date.time().minute(), 0);
        assert_eq!(date.time().second(), 0);
        
        let date = str2date("2021 08 07 13 00 00");
        assert_eq!(date.is_ok(), true);
        let date = date.unwrap();
        assert_eq!(date.date().year(), 2021);
        assert_eq!(date.date().month(), 08);
        assert_eq!(date.date().day(), 07);
        assert_eq!(date.time().hour(), 13);
        assert_eq!(date.time().minute(), 0);
        assert_eq!(date.time().second(), 0);
    }
}
