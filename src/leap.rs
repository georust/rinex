//! Describes `leap` second information, contained in `header` 
use thiserror::Error;
use crate::constellation;
use crate::constellation::Constellation;

/// `Leap` to describe leap seconds.
/// GLO = UTC = GPS - ΔtLS   
/// GPS = GPS = UTC + ΔtLS   
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Leap {
    /// current number
    leap: u32,
    /// ΔtLS : future or past leap second(s)  
    delta_tls: Option<u32>,
    /// week counter 
    week: Option<u32>,
    /// day counter
    day: Option<u32>,
    /// system time
    system: Option<Constellation>,
}

/// `Leap` parsing related errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse integer number")]
    ParseIntError(#[from] std::num::ParseIntError), 
    #[error("failed to identify time system")]
    TimeSystemError(#[from] constellation::Error),
}

impl Default for Leap {
    /// Builds a default (null) `Leap`
    fn default() -> Leap {
        Leap {
            leap: 0,
            delta_tls: None,
            week: None,
            day: None,
            system: None,
        }
    }
}

impl Leap {
    /// Builds a new `Leap` object to describe leap seconds
    pub fn new (leap: u32, delta_tls: Option<u32>, week: Option<u32>, day: Option<u32>, system: Option<Constellation>) -> Leap {
        Leap {
            leap,
            delta_tls,
            week,
            day,
            system,
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
                let (leap, rem) = s.split_at(6);
                let (tls, rem) = rem.split_at(6);
                let (week, rem) = rem.split_at(6);
                let (day, rem) = rem.split_at(6);
                let system = rem.trim();
                ls.leap = u32::from_str_radix(leap.trim(),10)?;
                ls.delta_tls = Some(u32::from_str_radix(tls.trim(),10)?);
                ls.week = Some(u32::from_str_radix(week.trim(),10)?);
                ls.day = Some(u32::from_str_radix(day.trim(),10)?);
                if system.eq("") {
                    ls.system = None
                } else {
                    ls.system = Some(constellation::Constellation::from_3_letter_code(system)?)
                }
            },
        }
        Ok(ls)
    }
}
