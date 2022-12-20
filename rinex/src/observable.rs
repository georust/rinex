use thiserror::Error;
use crate::carrier;
use crate::Carrier;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("failed to parse carrier code")]
    ParseCarrierError(#[from] carrier::Error),
    #[error("unknown observable")]
    UnknownObservable,
    #[error("malformed observable")]
    MalformedDescriptor,
}

/// Observable describes all possible observations,
/// forming Observation and Meteo RINEX epoch content.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observable {
    /// Carrier phase observation 
    Phase(Carrier, Option<String>),
    /// Doppler shift observation 
    Doppler(Carrier, Option<String>),
    /// SSI observation 
    SSI(Carrier, Option<String>),
    /// Pseudo range observation 
    PseudoRange(Carrier, Option<String>),
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
        Self::Phase(Carrier::default(), None)
    }
}

impl Observable {
    pub fn code(&self) -> Option<String> {
        match self {
            Self::Phase(_, c) | Self::Doppler(_, c) | Self::SSI(_, c) | Self::PseudoRange(_, c) => c.clone(),
            _ => None,
        }
    }
    pub fn carrier(&self) -> Option<Carrier> {
        match self {
            Self::Phase(c, _) | Self::Doppler(c, _) | Self::SSI(c, _) | Self::PseudoRange(c, _) => Some(*c),
            _ => None,
        }
    }
    pub fn wavelength(&self) -> Option<f64> {
        match self {
            Self::Phase(c, _) | Self::Doppler(c,  _) | Self::SSI(c, _) | Self::PseudoRange(c, _) => Some(c.carrier_wavelength()),
            _ => None,
        }
    }
    pub fn frequency(&self) -> Option<f64> {
        match self {
            Self::Phase(c, _) | Self::Doppler(c,  _) | Self::SSI(c, _) | Self::PseudoRange(c, _) => Some(c.carrier_frequency()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Observable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pressure =>  write!(f,"PR"),
            Self::Temperature =>  write!(f,"TD"),
            Self::HumidityRate =>  write!(f,"HR"),
            Self::ZenithWetDelay =>  write!(f,"ZW"),
            Self::ZenithDryDelay =>  write!(f,"ZD"),
            Self::ZenithTotalDelay =>  write!(f,"ZT"),
            Self::WindAzimuth =>  write!(f,"WD"),
            Self::WindSpeed =>  write!(f,"WS"),
            Self::RainIncrement =>  write!(f,"RI"),
            Self::HailIndicator =>  write!(f,"HI"),
            Self::SSI(c, code) => write!(f, "S{}{}", c, code.unwrap_or("".to_string())), 
            Self::Phase(c, code) => write!(f, "L{}{}", c, code.unwrap_or("".to_string())), 
            Self::Doppler(c, code) => write!(f, "D{}{}", c, code.unwrap_or("".to_string())), 
            Self::PseudoRange(c, code) => write!(f, "C{}{}", c, code.unwrap_or("".to_string())), 
        }
    }
}

impl std::str::FromStr for Observable {
	type Err = Error;
	fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.to_lowercase().trim();
		match content {
			"pr" => Ok(Self::Pressure),
			"td" => Ok(Self::Temperature),
			"hr" => Ok(Self::HumidityRate),
			"zw" => Ok(Self::ZenithWetDelay),
			"zd" => Ok(Self::ZenithDryDelay),
			"zt" => Ok(Self::ZenithTotalDelay),
			"wd" => Ok(Self::WindAzimuth),
			"ws" => Ok(Self::WindSpeed),
			"ri" => Ok(Self::RainIncrement),
			"hi" => Ok(Self::HailIndicator),
			_ => {
                let len = content.len();
                if len > 1 && len < 4 {
                    let carrier = Carrier::from_str(&content[1..2])?; 
                    let code: Option<String> = match len > 2 {
                        true => {
                            let code = &content[2..];
                            if carrier::KNOWN_CODES.contains(&code) {
                                Some(code.to_string())
                            } else {
                                None
                            }
                        },
                        false => None,
                    };
                    if content.starts_with("L")  {
                        Ok(Self::Phase(carrier, code))
                    } else if content.starts_with("C") {
                        Ok(Self::PseudoRange(carrier, code))
                    } else if content.starts_with("S") {
                        Ok(Self::SSI(carrier, code))
                    } else if content.starts_with("D") {
                        Ok(Self::Doppler(carrier, code))
                    } else {
                        Err(Error::UnknownObservable)
                    }
                } else {
                    Err(Error::MalformedDescriptor)
                }
			},
		}
	}
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_default() {
        let default = Observable::default();
        assert_eq!(default, Observable::from_str("L1").unwrap());
        assert_eq!(default, Observable::Phase(Carrier::L1));
    }
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
