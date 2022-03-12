//! `Epoch` description
use thiserror::Error;
use std::str::FromStr;
use chrono::{Datelike,Timelike};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum EpochFlag {
    Ok,
    PowerFailure,
    NewSiteOccupation,
    HeaderInformationFollows,
    ExternalEvent,
    CycleSlip,
}

impl Default for EpochFlag {
    fn default() -> EpochFlag { EpochFlag::Ok }
}

impl std::str::FromStr for EpochFlag {
    type Err = std::io::Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(EpochFlag::Ok),
            "1" => Ok(EpochFlag::PowerFailure),
            "3" => Ok(EpochFlag::NewSiteOccupation),
            "4" => Ok(EpochFlag::HeaderInformationFollows),
            "5" => Ok(EpochFlag::ExternalEvent),
            "6" => Ok(EpochFlag::CycleSlip),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid epoch flag value")),
        }
    }
}

/// An `Epoch` is an observation timestamp
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Epoch {
    /// `flag` validates or not this particular `epoch`
    pub flag: EpochFlag,
    /// `date`: sampling time stamp
    pub date: chrono::NaiveDateTime,
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
    pub fn new (date: chrono::NaiveDateTime, flag: EpochFlag) -> Epoch {
        Epoch { 
            date,
            flag,
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseDateError {
    #[error("failed to parse seconds field")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse y/m/d h:m fields")]
    ParseIntError(#[from] std::num::ParseIntError),
}

pub fn str2date (s: &str) -> Result<chrono::NaiveDateTime, ParseDateError> {
    let items : Vec<&str> = s.split_ascii_whitespace().collect();
    let (mut y,m,d,h,min,s) : (i32,u32,u32,u32,u32,f64) =
        (i32::from_str_radix(items[0],10)?,
         u32::from_str_radix(items[1],10)?,
         u32::from_str_radix(items[2],10)?,
         u32::from_str_radix(items[3],10)?,
         u32::from_str_radix(items[4],10)?,
         f64::from_str(items[5])?);
	// 2 digit nb case
    if y > 90 {
        y += 1900
    } else {
		y += 2000
	}
    Ok(chrono::NaiveDate::from_ymd(y,m,d)
        .and_hms(h,min,s as u32))
}
