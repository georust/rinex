//! Leap second counter
use crate::{
    prelude::{ParsingError, TimeScale},
    FormattingError,
};

use std::io::{BufWriter, Write};

/// `Leap` to describe leap seconds.
/// GLO = UTC = GPS - ΔtLS   
/// GPS = UTC + ΔtLS   
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Leap {
    /// Counter at the time of formatting
    pub leap: u32,
    /// ΔtLS : "future or past leap second(s)",
    /// actual number of leap seconds between GPS/GAL and GLO,
    /// or BDS and UTC.
    pub delta_tls: Option<u32>,
    /// Week counter
    pub week: Option<u32>,
    /// Days counter
    pub day: Option<u32>,
    /// Timescale definition
    pub timescale: Option<TimeScale>,
}

impl Leap {
    // Format [Leap] into [BufWriter]
    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        if let Some(delta) = &self.delta_tls {
            write!(
                w,
                "{:6}{:6}{:6}{:6} ",
                self.leap,
                delta,
                self.week.unwrap_or(0),
                self.day.unwrap_or(0),
            )?;
        } else {
            write!(w, "{:6}                   ", self.leap,)?;
        }

        if let Some(ts) = self.timescale {
            write!(w, "{:x}", ts)?;
        } else {
            write!(w, "   ")?;
        }

        writeln!(w, "                                LEAP SECONDS")?;

        Ok(())
    }

    // Copy and assign Delta [TLS]
    pub fn with_delta_tls(&self, delta_tls: u32) -> Self {
        let mut s = self.clone();
        s.delta_tls = Some(delta_tls);
        s
    }
    // Copy and assign Week counter
    pub fn with_week(&self, week: u32) -> Self {
        let mut s = self.clone();
        s.week = Some(week);
        s
    }
    // Copy and assign Day counter
    pub fn with_day(&self, day: u32) -> Self {
        let mut s = self.clone();
        s.day = Some(day);
        s
    }
    // Copy and assign Timescale
    pub fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.timescale = Some(ts);
        s
    }
}

impl std::str::FromStr for Leap {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Fields: leap, ΔtLS, week, day, then an optional time system that may
        // touch the day field ("7GPS"). Read them as tokens, so that short or
        // misaligned lines return an error instead of panicking.
        let s = s.trim();
        let numbers = s.trim_end_matches(|c: char| c.is_ascii_alphabetic());
        let system = s[numbers.len()..].trim();
        let items = numbers.split_ascii_whitespace().collect::<Vec<_>>();
        let parse = |item: &str| item.parse::<u32>().or(Err(ParsingError::LeapParsing));

        let mut ls = Leap::default();
        ls.leap = parse(items.first().ok_or(ParsingError::LeapParsing)?)?;

        // Only the complete format carries ΔtLS, week and day. With fewer fields
        // it is unclear which one is missing, so keep the current count only.
        if items.len() >= 4 {
            ls.delta_tls = Some(parse(items[1])?);
            ls.week = Some(parse(items[2])?);
            ls.day = Some(parse(items[3])?);
        }

        if !system.is_empty() {
            ls.timescale = Some(TimeScale::from_str(system)?);
        }

        Ok(ls)
    }
}

#[cfg(test)]
mod test {

    use crate::{
        prelude::{Leap, TimeScale},
        tests::formatting::Utf8Buffer,
    };

    use std::{io::BufWriter, str::FromStr};

    #[test]
    fn leap_second_basic() {
        assert_eq!(
            Leap::from_str("18").unwrap(),
            Leap {
                leap: 18,
                week: None,
                day: None,
                timescale: None,
                delta_tls: None,
            }
        );
    }
    #[test]
    fn leap_second_standard() {
        assert_eq!(
            Leap::from_str("18    18  2185     7").unwrap(),
            Leap {
                leap: 18,
                week: Some(2185),
                day: Some(7),
                timescale: None,
                delta_tls: Some(18),
            }
        );
    }

    #[test]
    fn leap_second_with_timescale() {
        assert_eq!(
            Leap::from_str("18    18  2185     7GPS").unwrap(),
            Leap {
                leap: 18,
                week: Some(2185),
                day: Some(7),
                delta_tls: Some(18),
                timescale: Some(TimeScale::GPST),
            }
        );
    }

    #[test]
    fn leap_second_missing_future_field() {
        // REDZ00POL_S_20242240000_01D_30S_MO.rnx header: "    18  2326     7"
        assert_eq!(
            Leap::from_str("18  2326     7").unwrap(),
            Leap {
                leap: 18,
                week: None,
                day: None,
                timescale: None,
                delta_tls: None,
            }
        );
    }

    #[test]
    fn leap_second_basic_with_timescale() {
        assert_eq!(
            Leap::from_str("18                  GPS").unwrap(),
            Leap {
                leap: 18,
                week: None,
                day: None,
                timescale: Some(TimeScale::GPST),
                delta_tls: None,
            }
        );
    }

    #[test]
    fn leap_second_single_digit() {
        assert_eq!(
            Leap::from_str("5     5   500     7").unwrap(),
            Leap {
                leap: 5,
                week: Some(500),
                day: Some(7),
                timescale: None,
                delta_tls: Some(5),
            }
        );
    }

    #[test]
    fn leap_second_malformed_is_an_error() {
        for s in ["", "   ", "GPS", "18 x 2185 7", "18    18  2185     7XYZ"] {
            assert!(Leap::from_str(s).is_err(), "{:?}", s);
        }
    }

    #[test]
    fn leap_formatting() {
        let leap = Leap::from_str("18").unwrap();
        let mut buf = BufWriter::new(Utf8Buffer::new(256));
        leap.format(&mut buf).unwrap();

        let ascii = buf.into_inner().unwrap().to_ascii_utf8();

        assert_eq!(
            &ascii,
            "    18                                                      LEAP SECONDS\n"
        );

        let leap = Leap::from_str("18    18  2185     7").unwrap();
        let mut buf = BufWriter::new(Utf8Buffer::new(256));
        leap.format(&mut buf).unwrap();

        let ascii = buf.into_inner().unwrap().to_ascii_utf8();

        assert_eq!(
            &ascii,
            "    18    18  2185     7                                    LEAP SECONDS\n"
        );
    }
}
