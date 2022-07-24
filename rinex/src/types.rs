//! `RINEX` files type description 
use thiserror::Error;
use crate::constellation;

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum Type {
    /// Describes Observation Data (OBS),
    /// Phase & Pseudo range measurements
    ObservationData, 
    /// Describes Navigation Data (NAV)
    /// Ephemeris file
    NavigationData,
    /// Describes Meteorological data (MET)
    MeteoData,
    /// Clock Data (CLK)
    ClockData,
    /// Ionsphere Maps (IONEX)
    /// allows creating Global Ionspheric Map (cartography)
    IonosphereMaps,
    /// Antenna Data (ATX or Antex) special RINEX format,
    /// with empty header and body describes several sets of
    /// Antenna characterization coefficients.
    /// No database is furnished by this lib (way too big).
    /// Users interested in such calibrations / conversions / calculations,
    /// should use this parser as a mean to extract the antenna coefficients solely
    AntennaData,
}

#[derive(Error, Debug)]
/// `Type` related errors
pub enum TypeError {
    #[error("Unknown RINEX type identifier \"{0}\"")]
    UnknownType(String),
}

impl Default for Type {
    /// Builds a default `Type`
    fn default() -> Type { Type::ObservationData }
}

impl Type {
    /// Converts `Self` to RINEX file format
    pub fn to_string (&self, constell: Option<constellation::Constellation>) -> String { 
        match *self {
            Type::ObservationData => String::from("OBSERVATION DATA"),
            Type::NavigationData => {
                match constell {
                    Some(constellation::Constellation::Glonass) => String::from("Glonass NAV"),
                    _ => String::from("NAV DATA"),
                }
            },
            Type::MeteoData => String::from("METEOROLOGICAL DATA"),
            Type::ClockData => String::from("CLOCK DATA"),
            Type::AntennaData => String::from("ANTEX"),
            Type::IonosphereMaps => String::from("IONOSPHERE MAPS"),
        }
    }
}

impl std::str::FromStr for Type {
    type Err = TypeError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") {
            Ok(Type::NavigationData)
        } else if s.contains("NAV DATA") {
            Ok(Type::NavigationData)
        } else if s.eq("OBSERVATION DATA") {
            Ok(Type::ObservationData)
        } else if s.eq("METEOROLOGICAL DATA") {
            Ok(Type::MeteoData)
        } else if s.eq("CLOCK DATA") || s.eq("C") {
            Ok(Type::ClockData)
        } else if s.eq("ANTEX") {
            Ok(Type::AntennaData)
        } else if s.eq("IONOSPHERE MAPS") {
            Ok(Type::IonosphereMaps)
        } else {
            Err(TypeError::UnknownType(String::from(s)))
        }
    }
}
