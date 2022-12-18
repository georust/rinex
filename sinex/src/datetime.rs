use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseDateTimeError {
    #[error("failed to parse YYYY:DDD")]
    DatetimeError(#[from] chrono::format::ParseError),
    #[error("failed to parse SSSSS")]
    ParseSecondsError(#[from] std::num::ParseFloatError),
}

pub fn parse_datetime(content: &str) -> Result<chrono::NaiveDateTime, ParseDateTimeError> {
    let ym = &content[0..8]; // "YYYY:DDD"
    let dt = chrono::NaiveDate::parse_from_str(&ym, "%Y:%j")?;
    let secs = &content[9..];
    let secs = f32::from_str(secs)?;
    let h = secs / 3600.0;
    let m = (secs - h * 3600.0) / 60.0;
    let s = secs - h * 3600.0 - m * 60.0;
    Ok(dt.and_hms(h as u32, m as u32, s as u32))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parsing() {
        let datetime = parse_datetime("2022:021:20823");
        assert_eq!(datetime.is_ok(), true);
        let datetime = parse_datetime("2022:009:00000");
        assert_eq!(datetime.is_ok(), true);
    }
}
