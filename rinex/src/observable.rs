#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observable {
    /// Carrier phase observation 
    Phase,
    /// Doppler shift observation 
    Doppler,
    /// SSI observation 
    SSI,
    /// Pseudo range observation 
    PseudoRange,
    /// Pressure observation in [mbar]
    Pressure,
    /// Dry temperature measurement in [°C]
    Temperature,
    /// Relative humidity measurement in [%]
    HumidityRate,
    /// Wet Zenith Path delay in [mm]
    ZenithWetDelay,
    /// Zenith path delay, dry component, in [mm]
    ZenithDryDelay,
    /// Total zenith path delay (dry + wet), in [mm]
    ZenithTotalDelay,
    /// Wind azimuth, from where the wind blows, in [°]
    WindAzimuth,
    /// Wind speed, in [m.s^-1]
    WindSpeed,
    /// Rain Increment, i.e., rain accumulation
    /// since previous measurement, [10th of mm]
    RainIncrement,
    /// Hail Indicator non zero, hail detected
    /// since last measurement
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

impl std::str::FromStr for Obserable {
	type Err = ObservableError;
	fn from_str(content: &str) -> Result<Self, Self::Err> {
		match content.to_lowercase().trim() {
			"pr" => Self::Pressure,
			"td" => Self::Temperature,
			"hr" => Self::HumidityRate,
			"zw" => Self::ZenithWetDelay,
			"zd" => Self::ZenithDryDelay,
			"zt" => Self::ZenithTotalDelay,
			"wd" => Self::WindAzimuth,
			"ws" => Self::WindSpeed,
			"ri" => Self::RainIncrement,
			"hi" => Self::HailIndicator,
			_ => {

			},
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
