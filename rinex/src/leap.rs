//! Describes `leap` second information, contained in `header` 
use thiserror::Error;
use hifitime::TimeScale;

/// `Leap` to describe leap seconds.
/// GLO = UTC = GPS - ΔtLS   
/// GPS = UTC + ΔtLS   
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leap {
    /// current number
    pub leap: u32,
    /// ΔtLS : "future or past leap second(s)", 
    /// actual number of leap seconds between GPS/GAL and GLO,
    /// or BDS and UTC.
    pub delta_tls: Option<u32>,
    /// weeks counter 
    pub week: Option<u32>,
    /// days counter
    pub day: Option<u32>,
    pub timescale: Option<TimeScale>,
}

/// `Leap` parsing related errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse leap integer number")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to parse leap timescale")]
    TimeScaleError(#[from] hifitime::Errors),
}

impl Default for Leap {
    /// Builds a default (null) `Leap`
    fn default() -> Leap {
        Leap {
            leap: 0,
            delta_tls: None,
            week: None,
            day: None,
            timescale: None,
        }
    }
}

impl Leap {
    /// Builds a new `Leap` object to describe leap seconds
    pub fn new (leap: u32, delta_tls: Option<u32>, week: Option<u32>, day: Option<u32>, timescale: Option<TimeScale>) -> Self {
        Self {
            leap,
            delta_tls,
            week,
            day,
            timescale,
        }
    }
}

impl std::str::FromStr for Leap {
    type Err = Error; 
    /// Builds `Leap` from standard RINEX descriptor
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ls = Leap::default();
        // leap second has two format
        let items: Vec<&str> = s.split_ascii_whitespace()
            .collect();
        match items.len() > 2 {
            false => {
                // [1] simple format: basic
                ls.leap = u32::from_str_radix(items[0].trim(),10)?
            },
            true => { 
                // [2] complex format: advanced infos
                let (leap, rem) = s.split_at(5);
                let (tls, rem) = rem.split_at(5);
                let (week, rem) = rem.split_at(5);
                let (day, rem) = rem.split_at(5);
                let system = rem.trim();
                ls.leap = u32::from_str_radix(leap.trim(),10)?;
                ls.delta_tls = Some(u32::from_str_radix(tls.trim(),10)?);
                ls.week = Some(u32::from_str_radix(week.trim(),10)?);
                ls.day = Some(u32::from_str_radix(day.trim(),10)?);
                if system.eq("") {
                    ls.timescale = None
                } else {
                    ls.timescale = Some(TimeScale::from_str(system)?)
                }
            },
        }
        Ok(ls)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    use hifitime::TimeScale;
    #[test]
    fn basic_format() {
        let content = "18";
        let leap = Leap::from_str(content); 
        assert_eq!(leap.is_ok(), true);
        let leap = leap.unwrap();
        assert_eq!(leap.leap, 18);
    }
    #[test]
    fn standard_format() {
        let content = "18    18  2185     7";
        let leap = Leap::from_str(content); 
        assert_eq!(leap.is_ok(), true);
        let leap = leap.unwrap();
        assert_eq!(leap.leap, 18);
        assert_eq!(leap.week, Some(2185));
        assert_eq!(leap.day, Some(7));
    }
    #[test]
    fn parse_with_timescale() {
        let content = "18    18  2185     7GPS";
        let leap = Leap::from_str(content); 
        assert_eq!(leap.is_ok(), true);
        let leap = leap.unwrap();
        assert_eq!(leap.leap, 18);
        assert_eq!(leap.week, Some(2185));
        assert_eq!(leap.day, Some(7));
    }
}
