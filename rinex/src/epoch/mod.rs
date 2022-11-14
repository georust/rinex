use thiserror::Error;
use std::str::FromStr;

use hifitime::Duration;
pub use hifitime::TimeScale;

use core::ops::{
    Add,
    AddAssign,
    Sub,
    SubAssign,
};

pub mod flag;
pub use flag::EpochFlag;

#[cfg(feature = "serde")]
use serde::{Serialize};

#[derive(Error, Debug)]
/// Epoch Parsing relate errors 
pub enum Error {
    #[error("expecting \"yyyy mm dd hh mm ss.ssss xx\" format")]
    FormatError, 
    #[error("failed to parse seconds + nanos")]
    SecsNanosError(#[from] std::num::ParseFloatError),
    #[error("failed to parse \"yyyy\" field")]
    YearError,
    #[error("failed to parse \"m\" month field")]
    MonthError,
    #[error("failed to parse \"d\" day field")]
    DayError,
    #[error("failed to parse \"hh\" field")]
    HoursError,
    #[error("failed to parse \"mm\" field")]
    MinutesError,
}

/// [hifitime::Epoch] high accuracy timestamp
/// (1 ns precision) with an [flag:EpochFlag] associated to it.
/// This precision is consistent with stringent Geodesics requirements.
/// Currently, the best precision in RINEX format is 100 ns 
/// for Observation RINEX. Default timescale is UTC 
/// with leap seconds are taken into account.
#[derive(Copy, Clone, Debug)]
#[derive(PartialOrd, Ord)]
#[derive(PartialEq, Eq, Hash)]
pub struct Epoch {
    epoch: hifitime::Epoch, 
    /// Flag describes sampling conditions and possible external events.
    /// Not all RINEX have this information, we default to "Sampling Ok"
    /// in this case.
    pub flag: flag::EpochFlag,
}

#[cfg(feature = "serde")]
impl Serialize for Epoch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{} {}", self.epoch, self.flag); 
        serializer.serialize_str(&s)
    }
}

impl Default for Epoch {
    fn default() -> Self {
        Self {
            flag: EpochFlag::default(),
            epoch: hifitime::Epoch::now()
                .expect("failed to retrieve system time"),
        }
    }
}

impl Sub for Epoch {
    type Output = Duration;
    fn sub(self, rhs: Self) -> Duration {
        self.epoch - rhs.epoch
    }
}

impl Sub<Duration> for Epoch {
    type Output = Self;
    fn sub(self, duration: Duration) -> Self {
        Self {
            epoch: self.epoch.set(self.epoch.to_duration() - duration),
            flag: self.flag,
        }
    }
}

impl SubAssign<Duration> for Epoch {
    fn sub_assign(&mut self, duration: Duration) {
        self.epoch -= duration; 
    }
}

impl Add<Duration> for Epoch {
    type Output = Self;
    fn add(self, duration: Duration) -> Self {
        Self {
            epoch: self.epoch.set(self.epoch.to_duration() + duration),
            flag: self.flag,
        }
    }
}

impl AddAssign<Duration> for Epoch {
    fn add_assign(&mut self, duration: Duration) {
        self.epoch += duration; 
    }
}

impl Epoch {
    /// Builds a new `Epoch` from given flag & timestamp in desired TimeScale
    pub fn new(epoch: hifitime::Epoch, flag: EpochFlag) -> Self {
        Self { 
            epoch,
            flag,
        }
    }
	/// Builds a current UTC instant description, with default flag
	pub fn now() -> Self {
		Self::default()
	}
	/// Builds an `epoch` with desired customized flag
	pub fn with_flag(&self, flag: EpochFlag) -> Self {
		Self {
			epoch: self.epoch,
			flag,
		}
	}
    /// Returns UTC date representation
    pub fn to_gregorian_utc(&self) -> (i32, u8, u8, u8, u8, u8, u32) {
        self.epoch.to_gregorian_utc()
    }
    /// Builds Self from given UTC date
    pub fn from_gregorian_utc(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: u8, nanos: u32) -> Self {
        Self {
            epoch: hifitime::Epoch::from_gregorian_utc(year, month, day, hour, minute, second, nanos),
            flag: EpochFlag::default(),
        }
    }
    /// Returns timescale in use
    pub fn timescale(&self) -> TimeScale {
        self.epoch.time_scale
    }
}

impl std::fmt::Display for Epoch {
    /// Default formatter applies to Observation RINEX only
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (y, m, d, hh, mm, ss, nanos) = self.to_gregorian_utc();
        write!(f,
            "{:04} {:>2} {:>2} {:>2} {:>2} {:>2}.{:07}  {}",
            y, m, d, hh, mm, ss, nanos, self.flag)
    }
}

impl std::fmt::LowerExp for Epoch {
    /// LowerExp "e" applies to old formats like NAV V2 that omit the "flag" 
    /// and accuracy is 0.1 sec
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (y, m, d, hh, mm, ss, ns) = self.to_gregorian_utc();
        write!(f, 
            "{:04} {:>2} {:>2} {:>2} {:>2} {:>2}.{:1}",
            y, m, d, hh, mm, ss, ns)
    }
}

impl std::fmt::UpperExp for Epoch {
    /// UpperExp "E" applies to modern formats like NAV V3/V4 that omit the "flag"
    /// and accuracy is 1 sec
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (y, m, d, hh, mm, ss, _) = self.epoch.to_gregorian_utc();
        write!(f,
            "{:04} {:>2} {:>2} {:>2} {:>2} {:>2}",
            y, m, d, hh, mm, ss)
    }
}

impl FromStr for Epoch {
    type Err = Error;
    /// Parses an [Epoch] from all known RINEX formats
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items : Vec<&str> = s.split_ascii_whitespace()
            .collect();
        if items.len() != 6 {
            if items.len() != 7 {
                return Err(Error::FormatError)
            }
        }
        if let Ok(mut y) = i32::from_str_radix(items[0], 10) {
            if y < 100 { // old rinex -__-
                if y > 90 {
                    y += 1900;
                } else {
                    y += 2000;
                }
            }
            if let Ok(m) = u8::from_str_radix(items[1], 10) {
                if let Ok(d) = u8::from_str_radix(items[2], 10) {
                    if let Ok(hh) = u8::from_str_radix(items[3], 10) {
                        if let Ok(mm) = u8::from_str_radix(items[4], 10) {
                            let ss = f64::from_str(items[5].trim())?;
                            let second = ss.trunc() as u8;
                            let nanos = (ss.fract() * 10.0) as u32;
                            let mut e = Self::from_gregorian_utc(y, m, d, hh, mm, second, nanos);
                            if items.len() == 7 { // flag exists
                                if let Ok(flag) = EpochFlag::from_str(items[6].trim()) {
                                    e = e.with_flag(flag);
                                }
                            }
                            Ok(e)
                        } else {
                            Err(Error::MinutesError)
                        }
                    } else {
                        Err(Error::HoursError)
                    }
                } else {
                    Err(Error::DayError)
                }
            } else {
                Err(Error::MonthError)
            }
        } else {
            Err(Error::YearError)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_nav_v2() {
        let e = Epoch::from_str("20 12 31 23 45  0.0");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 12);
        assert_eq!(d, 31);
        assert_eq!(hh, 23);
        assert_eq!(mm, 45);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
        let e = Epoch::from_str("21  1  1 16 15  0.0");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 16);
        assert_eq!(mm, 15);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
    }
    #[test]
    fn test_parse_nav_v3() {
        let e = Epoch::from_str("2021 01 01 00 00 00 ");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
        
        let e = Epoch::from_str("2021 01 01 09 45 00 ");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2021);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
        assert_eq!(hh, 09);
        assert_eq!(mm, 45);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
        
        let e = Epoch::from_str("2020 06 25 00 00 00");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 6);
        assert_eq!(d, 25);
        assert_eq!(hh, 00);
        assert_eq!(mm, 00);
        assert_eq!(ss, 0);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
        
        let e = Epoch::from_str("2020 06 25 09 49 04");
        assert_eq!(e.is_ok(), true);
        let e = e.unwrap();
        let (y, m, d, hh, mm, ss, ns) = e.to_gregorian_utc();
        assert_eq!(y, 2020);
        assert_eq!(m, 6);
        assert_eq!(d, 25);
        assert_eq!(hh, 09);
        assert_eq!(mm, 49);
        assert_eq!(ss, 04);
        assert_eq!(ns, 0);
        assert_eq!(e.timescale(), TimeScale::UTC);
        assert_eq!(e.flag, EpochFlag::Ok);
    }
}
