//! `Epoch` is an observation timestamp with
//! a `flag` associated to it
use thiserror::Error;
use std::str::FromStr;
use chrono::{Datelike,Timelike};

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

/// `EpochFlag` validates an epoch, 
/// or describes possible events that occurred
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum EpochFlag {
    /// Epoch is sane
    Ok,
    /// Power failure since previous epoch
    PowerFailure,
    /// Antenna is being moved at current epoch
    AntennaBeingMoved,
    /// Site has changed, received has moved since last epoch
    NewSiteOccupation,
    /// New information to come after this epoch
    HeaderInformationFollows,
    /// External event - significant event in this epoch
    ExternalEvent,
    /// Cycle slip at this epoch
    CycleSlip,
}

impl Default for EpochFlag {
    fn default() -> EpochFlag { EpochFlag::Ok }
}

impl EpochFlag {
    /// Returns True if self is a valid epoch
    pub fn is_ok (self) -> bool { self == EpochFlag::Ok }
}

impl std::str::FromStr for EpochFlag {
    type Err = std::io::Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(EpochFlag::Ok),
            "1" => Ok(EpochFlag::PowerFailure),
            "2" => Ok(EpochFlag::AntennaBeingMoved),
            "3" => Ok(EpochFlag::NewSiteOccupation),
            "4" => Ok(EpochFlag::HeaderInformationFollows),
            "5" => Ok(EpochFlag::ExternalEvent),
            "6" => Ok(EpochFlag::CycleSlip),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid epoch flag value")),
        }
    }
}

impl std::fmt::Display for EpochFlag {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EpochFlag::Ok => f.write_str("0"),
            EpochFlag::PowerFailure => f.write_str("1"),
            EpochFlag::AntennaBeingMoved => f.write_str("2"),
            EpochFlag::NewSiteOccupation => f.write_str("3"),
            EpochFlag::HeaderInformationFollows => f.write_str("4"),
            EpochFlag::ExternalEvent => f.write_str("5"),
            EpochFlag::CycleSlip => f.write_str("6"),
        }
    }
}

/// An `Epoch` is an observation timestamp 
/// with an [epoch::EpochFlag] associated to it
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Epoch {
    /// Sampling timestamp
    pub date: chrono::NaiveDateTime,
    /// Associated flag
    pub flag: EpochFlag,
}

#[cfg(feature = "with-serde")]
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

/*impl std::fmt::Display for Epoch {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("salut")
    }
}*/

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
    /// Converts self to string in standard format,
    /// this is mainly used in file production [rinex::to_file]
    pub fn to_string (&self) -> &str { "TODO" }
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
