//! `RINEX` types description
use crate::header::ParsingError;
use crate::prelude::Constellation;

/// Describes all known `RINEX` file types
#[derive(Default, Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Type {
    /// Describes Observation Data (OBS),
    /// Phase & Pseudo range measurements
    #[default]
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

impl std::fmt::Display for Type {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::ObservationData => write!(fmt, "OBS DATA"),
            Self::NavigationData => write!(fmt, "NAVIGATION DATA"),
            Self::MeteoData => write!(fmt, "METEO DATA"),
            Self::ClockData => write!(fmt, "CLOCK DATA"),
            Self::AntennaData => write!(fmt, "ANTEX"),
            Self::IonosphereMaps => write!(fmt, "IONOSPHERE MAPS"),
        }
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
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("NAVIGATION DATA") || s.contains("NAV DATA") {
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
            Err(ParsingError::TypeParsing(String::from(s)))
        }
    }
}
