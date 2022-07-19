//! Meteo observable codes
use strum_macros::EnumString;

/// Known Meteo Observables
#[derive(Debug, Clone)]
#[derive(PartialEq, PartialOrd)]
#[derive(Hash, Eq)]
#[derive(EnumString)]
pub enum Observable {
    /// Pressure observation in [mbar]
    #[strum(serialize = "PR", deserialize = "PR")]
    Pressure,
    /// Dry temperature measurement in [°C]
    #[strum(serialize = "TD", deserialize = "TD")]
    Temperature,
/*
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
    HailIndicator, */
}

impl Default for Observable {
    fn default() -> Self {
        Self::Temperature
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_parser() {
        let obs = Observable::from_str("PR");
        assert_eq!(obs.is_ok(), true);
    }
}
