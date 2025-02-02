//! header line #2 parsing helper

use crate::ParsingError;
use hifitime::Duration;

pub(crate) fn is_header_line2(content: &str) -> bool {
    content.starts_with("##")
}

pub(crate) struct Line2 {
    pub week_counter: (u32, f64),
    pub epoch_interval: Duration,
    pub mjd: (u32, f64),
}

impl std::str::FromStr for Line2 {
    type Err = ParsingError;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.len() != 60 {
            return Err(ParsingError::MalformedH2);
        }
        let mut mjd = (0_u32, 0.0_f64);
        let mut week_counter = (0_u32, 0.0_f64);

        week_counter.0 = u32::from_str(line[2..7].trim())
            .or(Err(ParsingError::WeekCounter(line[2..7].to_string())))?;

        week_counter.1 = f64::from_str(line[7..23].trim())
            .or(Err(ParsingError::WeekCounter(line[7..23].to_string())))?;

        let dt = f64::from_str(line[24..38].trim())
            .or(Err(ParsingError::EpochInterval(line[24..38].to_string())))?;

        mjd.0 = u32::from_str(line[38..44].trim())
            .or(Err(ParsingError::Mjd(line[38..44].to_string())))?;

        mjd.1 =
            f64::from_str(line[44..].trim()).or(Err(ParsingError::Mjd(line[44..].to_string())))?;

        Ok(Self {
            week_counter,
            epoch_interval: Duration::from_seconds(dt),
            mjd,
        })
    }
}

impl Line2 {
    pub(crate) fn to_parts(&self) -> ((u32, f64), Duration, (u32, f64)) {
        (self.week_counter, self.epoch_interval, self.mjd)
    }
}

#[cfg(test)]
mod test {
    use super::Line2;
    use std::str::FromStr;

    #[test]
    fn test_line2_parsing() {
        for (line, week_counter, week_sow, epoch_interval, mjd, mjd_fract) in [(
            "##  887      0.00000000   900.00000000 50453 0.0000000000000",
            887,
            0.0,
            900.0,
            50453,
            0.0,
        )] {
            let line2 = Line2::from_str(&line).unwrap();

            assert_eq!(line2.week_counter.0, week_counter);
            assert_eq!(line2.week_counter.1, week_sow);
            assert_eq!(line2.mjd.0, mjd);
            assert_eq!(line2.mjd.1, mjd_fract);
            assert_eq!(line2.epoch_interval.to_seconds(), epoch_interval);
        }
    }
}
