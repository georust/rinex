use std::str::FromStr;
use thiserror::Error;
use rinex::constellation::Constellation;

pub mod bias;
pub mod receiver;

fn is_comment (line: &str) -> bool {
    line.starts_with("*")
}

fn section_start (line: &str) -> Option<String> {
    if line.starts_with("+") {
        Some(line[1..].to_string())
    } else {
        None
    }
}

fn section_end (line: &str) -> Option<String> {
    if line.starts_with("-") {
        Some(line[1..].to_string())
    } else {
        None
    }
}

#[derive(Debug, Error)]
pub enum ParseDateTimeError {
    #[error("failed to parse YYYY:DDD")]
    DatetimeError(#[from] chrono::format::ParseError),
    #[error("failed to parse SSSSS")]
    ParseSecondsError(#[from] std::num::ParseFloatError),

}

fn parse_datetime (content: &str) -> Result<chrono::NaiveDateTime, ParseDateTimeError> {
    let ym = &content[0..8]; // "YYYY:DDD"
    let dt = chrono::NaiveDate::parse_from_str(&ym, "%Y:%j")?;
    let secs = &content[9..];
    let secs = f32::from_str(secs)?;
    let h = secs /3600.0;
    let m = (secs - h*3600.0)/60.0;
    let s = secs - h*3600.0 - m*60.0;
    Ok(dt.and_hms(h as u32, m as u32, s as u32))
}

pub struct Header {
    pub input: String,
    pub output: String,
    pub contact: String,
    pub hardware: String,
    pub software: String,
    pub reference_frame: String,
}

pub enum DataType {
    ObsSampling,
    ParmeterSpacing,
    DeterminationMethod,
    BiasMode,
    TimeSystem,
    ReceiverClockRef,
    SatelliteClockReferenceObs,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_receiver() {
        //"STATION__ C GROUP____ DATA_START____ DATA_END______ RECEIVER_TYPE_______ RECEIVER_FIRMWARE___"
        let rcvr = Receiver::from_str(
        "MAO0      G @MP0      2015:276:00000 2015:276:86399 JAVAD TRE-G3TH DELTA 3.6.4");
        assert_eq!(rcvr.is_ok(), true);
        let rcvr = rcvr.unwrap();
        println!("{:?}", rcvr);
        assert_eq!(rcvr.station, "MAO0");
        assert_eq!(rcvr.group, "@MP0");
        assert_eq!(rcvr.firmware, "3.6.4");
        assert_eq!(rcvr.rtype, "JAVAD TRE-G3TH DELTA");
    }
}
