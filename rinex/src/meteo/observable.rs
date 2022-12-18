//! Meteo observable codes
use strum_macros::EnumString;

/// Meteo Observables
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observable {
    /// Pressure observation in [mbar]
    #[strum(serialize = "PR")]
    Pressure,
    /// Dry temperature measurement in [°C]
    #[strum(serialize = "TD")]
    Temperature,
    /// Relative humidity measurement in [%]
    #[strum(serialize = "HR")]
    HumidityRate,
    /// Wet Zenith Path delay in [mm]
    #[strum(serialize = "ZW")]
    ZenithWetDelay,
    /// Zenith path delay, dry component, in [mm]
    #[strum(serialize = "ZD")]
    ZenithDryDelay,
    /// Total zenith path delay (dry + wet), in [mm]
    #[strum(serialize = "ZT")]
    ZenithTotalDelay,
    /// Wind azimuth, from where the wind blows, in [°]
    #[strum(serialize = "WD")]
    WindAzimuth,
    /// Wind speed, in [m.s^-1]
    #[strum(serialize = "WS")]
    WindSpeed,
    /// Rain Increment, i.e., rain accumulation
    /// since previous measurement, [10th of mm]
    #[strum(serialize = "RI")]
    RainIncrement,
    /// Hail Indicator non zero, hail detected
    /// since last measurement
    #[strum(serialize = "HI")]
    HailIndicator,
}

impl Default for Observable {
    fn default() -> Self {
        Self::Temperature
    }
}

impl std::fmt::Display for Observable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pressure => "PR".fmt(f),
            Self::Temperature => "TD".fmt(f),
            Self::HumidityRate => "HR".fmt(f),
            Self::ZenithWetDelay => "ZW".fmt(f),
            Self::ZenithDryDelay => "ZD".fmt(f),
            Self::ZenithTotalDelay => "ZT".fmt(f),
            Self::WindAzimuth => "WD".fmt(f),
            Self::WindSpeed => "WS".fmt(f),
            Self::RainIncrement => "RI".fmt(f),
            Self::HailIndicator => "HI".fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_parser() {
        let obs = Observable::from_str("PR");
        assert_eq!(obs.is_ok(), true);
        let obs = obs.unwrap();
        assert_eq!(obs, Observable::Pressure);
        assert_eq!(obs.to_string(), "PR");

        let obs = Observable::from_str("WS");
        assert_eq!(obs.is_ok(), true);
        let obs = obs.unwrap();
        assert_eq!(obs, Observable::WindSpeed);
        assert_eq!(obs.to_string(), "WS");

        let obs = Observable::from_str("Wa");
        assert_eq!(obs.is_ok(), false);
    }
}
