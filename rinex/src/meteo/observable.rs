//! Meteo observable codes
use strum_macros::EnumString;

/// Known Meteo Observables
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Hash, Eq)]
#[derive(EnumString)]
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
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pressure => f.write_str("PR"),
            Self::Temperature => f.write_str("TD"),
            Self::HumidityRate => f.write_str("HR"),
            Self::ZenithWetDelay => f.write_str("ZW"),
            Self::ZenithDryDelay => f.write_str("ZD"),
            Self::ZenithTotalDelay => f.write_str("ZT"),
            Self::WindAzimuth => f.write_str("WD"),
            Self::WindSpeed => f.write_str("WS"),
            Self::RainIncrement => f.write_str("RI"),
            Self::HailIndicator => f.write_str("HI"),
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
