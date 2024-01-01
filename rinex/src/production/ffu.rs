use super::Error;
use hifitime::{Duration, Unit};

#[derive(Debug, Clone, PartialEq)]
pub struct FFU {
    /// Sample rate
    pub val: u32,
    /// Period unit
    pub unit: Unit,
}

impl Default for FFU {
    fn default() -> Self {
        Self {
            val: 30,
            unit: Unit::Second,
        }
    }
}

impl std::fmt::Display for FFU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.unit {
            Unit::Minute => write!(f, "{:02}M", self.val),
            Unit::Hour => write!(f, "{:02}H", self.val),
            Unit::Day => write!(f, "{:02}D", self.val),
            Unit::Second | _ => write!(f, "{:02}S", self.val),
        }
    }
}

impl std::str::FromStr for FFU {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 3 {
            return Err(Error::InvalidFFU);
        }
        let val = s[..2].parse::<u32>().map_err(|_| Error::InvalidFFU)?;
        let unit = match s.chars().nth(2) {
            Some('S') => Unit::Second,
            Some('M') => Unit::Minute,
            Some('H') => Unit::Hour,
            Some('D') => Unit::Day,
            _ => return Err(Error::InvalidFFU),
        };
        Ok(Self { val, unit })
    }
}

impl From<Duration> for FFU {
    fn from(dur: Duration) -> Self {
        let secs = dur.to_seconds().round() as u32;
        if secs < 60 {
            Self {
                val: secs,
                unit: Unit::Second,
            }
        } else if secs < 3_600 {
            Self {
                val: secs / 60,
                unit: Unit::Minute,
            }
        } else if secs < 86_400 {
            Self {
                val: secs / 3_600,
                unit: Unit::Hour,
            }
        } else {
            Self {
                val: secs / 86_400,
                unit: Unit::Day,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::FFU;
    use hifitime::Unit;
    use std::str::FromStr;
    #[test]
    fn ffu_parsing() {
        for (desc, expected) in [
            (
                "30S",
                FFU {
                    val: 30,
                    unit: Unit::Second,
                },
            ),
            (
                "01M",
                FFU {
                    val: 1,
                    unit: Unit::Minute,
                },
            ),
            (
                "15M",
                FFU {
                    val: 15,
                    unit: Unit::Minute,
                },
            ),
            (
                "30M",
                FFU {
                    val: 30,
                    unit: Unit::Minute,
                },
            ),
            (
                "01H",
                FFU {
                    val: 1,
                    unit: Unit::Hour,
                },
            ),
            (
                "04H",
                FFU {
                    val: 4,
                    unit: Unit::Hour,
                },
            ),
            (
                "08H",
                FFU {
                    val: 8,
                    unit: Unit::Hour,
                },
            ),
            (
                "01D",
                FFU {
                    val: 1,
                    unit: Unit::Day,
                },
            ),
            (
                "07D",
                FFU {
                    val: 7,
                    unit: Unit::Day,
                },
            ),
        ] {
            let ffu = FFU::from_str(desc).unwrap();
            assert_eq!(ffu, expected);
        }
    }
}
