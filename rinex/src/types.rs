//! `RINEX` files type description
use super::Constellation;
use thiserror::Error;

/// Describes all known `RINEX` file types
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Type {
    /// Describes Observation Data (OBS),
    /// Phase & Pseudo range measurements
    ObservationData,
    /// Describes Navigation Data (NAV)
    /// Ephemeris data, and other possible
    /// modern declinations
    NavigationData,
    /// Describes Meteorological data (MET)
    MeteoData,
    /// Clock Data (CLK)
    ClockData,
    /// Ionosphere Maps (IONEX)
    /// contains list of TEC Maps.
    IonosphereMaps,
    /// Antenna Data (ATX or Antex) special RINEX format,
    /// contains sets of Antenna characterization coefficients.
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
    fn default() -> Type {
        Self::ObservationData
    }
}

impl Type {
    /// Converts `Self` to RINEX file format
    pub fn to_string(&self, constell: Option<Constellation>) -> String {
        match *self {
            Self::ObservationData => String::from("OBSERVATION DATA"),
            Self::NavigationData => match constell {
                Some(Constellation::Glonass) => String::from("Glonass NAV"),
                _ => String::from("NAV DATA"),
            },
            Self::MeteoData => String::from("METEOROLOGICAL DATA"),
            Self::ClockData => String::from("CLOCK DATA"),
            Self::AntennaData => String::from("ANTEX"),
            Self::IonosphereMaps => String::from("IONOSPHERE MAPS"),
        }
    }
}

impl std::str::FromStr for Type {
    type Err = TypeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") {
            Ok(Self::NavigationData)
        } else if s.contains("NAV DATA") {
            Ok(Self::NavigationData)
        } else if s.eq("OBSERVATION DATA") {
            Ok(Self::ObservationData)
        } else if s.eq("METEOROLOGICAL DATA") {
            Ok(Self::MeteoData)
        } else if s.eq("CLOCK DATA") || s.eq("C") {
            Ok(Self::ClockData)
        } else if s.eq("ANTEX") {
            Ok(Self::AntennaData)
        } else if s.eq("IONOSPHERE MAPS") {
            Ok(Self::IonosphereMaps)
        } else {
            Err(TypeError::UnknownType(String::from(s)))
        }
    }
}
