//! Product type and report classification
use rinex::prelude::RinexType;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum ProductType {
    /// GNSS carrier signal observation in the form
    /// of Observation RINEX data.
    Observation,
    /// Meteo sensors data wrapped as Meteo RINEX files.
    MeteoObservation,
    /// DORIS measurements wrapped as special RINEX observation file.
    DorisRinex,
    /// Broadcast Navigation message as contained in
    /// Navigation RINEX files.
    BroadcastNavigation,
    #[cfg(feature = "sp3")]
    /// High precision orbits wrapped in SP3 files.
    HighPrecisionOrbit,
    /// High precision orbital attitudes wrapped in Clock RINEX files.
    HighPrecisionClock,
    /// Antenna calibration information wrapped in ANTEX special RINEX files.
    ANTEX,
    /// Precise Ionosphere state wrapped in IONEX special RINEX files.
    IONEX,
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Observation => write!(f, "Observation"),
            Self::MeteoObservation => write!(f, "Meteo"),
            Self::BroadcastNavigation => write!(f, "Broadcast Navigation"),
            Self::HighPrecisionOrbit => write!(f, "High Precision Orbit (SP3)"),
            Self::HighPrecisionClock => write!(f, "High Precision Clock"),
            Self::ANTEX => write!(f, "ANTEX"),
            Self::IONEX => write!(f, "IONEX"),
            Self::DorisRinex => write!(f, "DORIS RINEX"),
        }
    }
}

impl From<RinexType> for ProductType {
    fn from(rt: RinexType) -> Self {
        match rt {
            RinexType::ObservationData => Self::Observation,
            RinexType::NavigationData => Self::BroadcastNavigation,
            RinexType::MeteoData => Self::MeteoObservation,
            RinexType::ClockData => Self::HighPrecisionClock,
            RinexType::IonosphereMaps => Self::IONEX,
            RinexType::AntennaData => Self::ANTEX,
            RinexType::DORIS => Self::DorisRinex,
        }
    }
}
