use thiserror::Error;
//use std::str::FromStr;

/// `MeteoDataCodeError`
#[derive(Error, Debug)]
pub enum MeteoDataCodeError {
    #[error("unknown meteo data code \"{0}\"")]
    UnknownMeteoDataCode(String),
}

/// `MeteoDataCode` describes `Meteo` data
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MeteoDataCode {
    Temperature,
    Moisture,
    Pressure,
}

impl std::str::FromStr for MeteoDataCode {
    type Err = MeteoDataCodeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("PR") {
            Ok(MeteoDataCode::Pressure)
        } else if s.eq("TD") {
            Ok(MeteoDataCode::Temperature)
        } else if s.eq("HR") {
            Ok(MeteoDataCode::Moisture)
        } else {
            Err(MeteoDataCodeError::UnknownMeteoDataCode(s.to_string()))
        }
    }
}

impl Default for MeteoDataCode {
    fn default() -> MeteoDataCode { MeteoDataCode::Temperature }
}

/*pub fn build_meteo_entry (content: &str, header: &RinexHeader)
    -> Result<Vec<MeteoData>, RecordItemError> 
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
}*/
