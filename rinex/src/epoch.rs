//! `Epoch` is an observation timestamp with
//! a `flag` associated to it
use thiserror::Error;
use std::str::FromStr;
use chrono::{Datelike,Timelike};

#[cfg(feature = "with-serde")]
use serde::{Serialize, Deserialize};

/// `EpochFlag` validates or describes events
/// that occured during an `epoch`
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
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
            EpochFlag::Ok => f.write_str("Ok"),
            EpochFlag::PowerFailure => f.write_str("PowerFailure"),
            EpochFlag::AntennaBeingMoved => f.write_str("AntennaBeingMoved"),
            EpochFlag::NewSiteOccupation => f.write_str("NewSiteOccupation"),
            EpochFlag::HeaderInformationFollows => f.write_str("HeaderInformationFollows"),
            EpochFlag::ExternalEvent => f.write_str("ExternalEvent"),
            EpochFlag::CycleSlip => f.write_str("CycleSlip"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// An `Epoch` is an observation timestamp associated
/// to an `EpochFlag`
pub struct Epoch {
    /// `date`: sampling time stamp
    pub date: chrono::NaiveDateTime,
    /// `flag` validates or not this particular `epoch`
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
    /// Builds a new `Epoch` structure using given
    /// timestamp and `EpochFlag` 
    pub fn new (date: chrono::NaiveDateTime, flag: EpochFlag) -> Epoch {
        Epoch { 
            date,
            flag,
        }
    }
    pub fn to_string (&self) -> &str { "hello" }
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
pub fn str2date (s: &str) -> Result<chrono::NaiveDateTime, ParseDateError> {
    let items : Vec<&str> = s.split_ascii_whitespace().collect();
    if items.len() != 6 {
        return Err(ParseDateError::FormatMismatch)
    }
    let (mut y,m,d,h,min,s) : (i32,u32,u32,u32,u32,f64) =
        (i32::from_str_radix(items[0],10)?,
         u32::from_str_radix(items[1],10)?,
         u32::from_str_radix(items[2],10)?,
         u32::from_str_radix(items[3],10)?,
         u32::from_str_radix(items[4],10)?,
         f64::from_str(items[5])?);
	if y < 100 { // 2 digit nb case
    	if y > 90 { // old rinex
        	y += 1900
    	} else {
			y += 2000
		}
	}
    Ok(chrono::NaiveDate::from_ymd(y,m,d)
        .and_hms(h,min,s as u32))
}
