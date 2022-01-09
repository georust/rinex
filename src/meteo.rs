use thiserror::Error;
use std::str::FromStr;

use crate::header::RinexHeader;
use crate::record::{Epoch, RecordItemError};

/// MeteoObservationType related errors
#[derive(Error, Debug)]
pub enum MeteoObservationTypeError {
    #[error("unknown type of meteo obs \"{0}\"")]
    UnknownMeteoObsType(String),
}

/// Describes different kind of `Meteo` Observations
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MeteoObservationType {
    Temperature,
    Moisture,
    Pressure,
}

impl std::str::FromStr for MeteoObservationType {
    type Err = MeteoObservationTypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("PR") {
            Ok(MeteoObservationType::Pressure)
        } else if s.eq("TD") {
            Ok(MeteoObservationType::Temperature)
        } else if s.eq("HR") {
            Ok(MeteoObservationType::Moisture)
        } else {
            Err(MeteoObservationTypeError::UnknownMeteoObsType(s.to_string()))
        }
    }
}

impl Default for MeteoObservationType {
    fn default() -> MeteoObservationType { MeteoObservationType::Temperature }
}

pub fn build_meteo_entry (content: &str, header: &RinexHeader)
    -> Result<(Epoch,Vec<f64>), RecordItemError> 
{
    let measurements: Vec<f64> = Vec::with_capacity(4);
    let (e_str, rem) = content.split_at(23);
    let e = Epoch::from_str(e_str.trim())?;
    let items: Vec<&str> = rem.split_ascii_whitespace()
        .collect();
    //for i in 0..items.len() {
    //    measurements.push(f64::from_str(items[i].trim())?)
    //}
    Ok((e, measurements))
}
