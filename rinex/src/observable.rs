use crate::{carrier, Carrier, Constellation};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
/// Observable Parsing errors
pub enum ParsingError {
    #[error("unknown observable \"{0}\"")]
    UnknownObservable(String),
    #[error("malformed observable \"{0}\"")]
    MalformedDescriptor(String),
}

/// Observable describes all possible observations,
/// forming Observation and Meteo RINEX epoch content.
#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Ord, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observable {
    /// Carrier phase observation
    Phase(String),
    /// Doppler shift observation
    Doppler(String),
    /// SSI observation
    SSI(String),
    /// Pseudo range observation
    PseudoRange(String),
    /// Pressure observation in hPa
    Pressure,
    /// Dry temperature measurement in Celcius degrees
    Temperature,
    /// Relative humidity measurement in %
    HumidityRate,
    /// Wet Zenith Path delay in mm
    ZenithWetDelay,
    /// Zenith path delay, dry component in mm
    ZenithDryDelay,
    /// Total zenith path delay (dry + wet) in mm
    ZenithTotalDelay,
    /// Wind direction azimuth in degrees
    WindDirection,
    /// Wind speed in m.s⁻¹
    WindSpeed,
    /// Rain Increment: rain accumulation
    /// since previous measurement, in 10th of mm
    RainIncrement,
    /// Hail Indicator
    HailIndicator,
}

impl Default for Observable {
    fn default() -> Self {
        Self::Phase("L1C".to_string())
    }
}

impl Observable {
    pub fn is_phase_observable(&self) -> bool {
        match self {
            Self::Phase(_) => true,
            _ => false,
        }
    }
    pub fn is_pseudorange_observable(&self) -> bool {
        match self {
            Self::PseudoRange(_) => true,
            _ => false,
        }
    }
    pub fn is_doppler_observable(&self) -> bool {
        match self {
            Self::Doppler(_) => true,
            _ => false,
        }
    }
    pub fn is_ssi_observable(&self) -> bool {
        match self {
            Self::SSI(_) => true,
            _ => false,
        }
    }
    pub fn code(&self) -> Option<String> {
        match self {
            Self::Phase(c) | Self::Doppler(c) | Self::SSI(c) | Self::PseudoRange(c) => {
                if c.len() == 3 {
                    Some(c[1..].to_string())
                } else {
                    None
                }
            },
            _ => None,
        }
    }
    pub fn carrier(&self, c: Constellation) -> Result<Carrier, carrier::Error> {
        Carrier::from_observable(c, self)
    }
    /// Returns the code length (repetition period), expressed in seconds,
    /// of self: a valid Pseudo Range observable. This is not intended to be used
    /// on phase observables, although they are also determined from PRN codes.
    /// This is mostly used in fractional pseudo range determination.
    pub fn code_length(&self, c: Constellation) -> Option<f64> {
        match c {
            Constellation::GPS => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(20.0E-3_f64),
                            "C1C" => Some(1.0_f64), // TODO
                            "C1L" => Some(1.0_f64), // TODO
                            "C1X" => Some(1.0_f64), // TODO
                            "C1P" => Some(1.0_f64), // TODO,
                            "C1W" => Some(1.0_f64), // TODO
                            "C1Y" => Some(1.0_f64), // TODO
                            "C1M" => Some(1.0_f64), // TODO
                            "C2" => Some(1.0_f64),  //TODO
                            "C2D" => Some(1.0_f64), //TODO
                            "C2S" => Some(1.0_f64), //TODO
                            "C2L" => Some(1.0_f64), //TODO
                            "C2X" => Some(1.0_f64), //TODO
                            "C2P" => Some(1.0_f64), //TODO
                            "C2W" => Some(1.0_f64), //TODO
                            "C2Y" => Some(1.0_f64), //TODO
                            "C2M" => Some(1.0_f64), //TODO
                            _ => None,              // does not apply
                        }
                    },
                    _ => None, // invalid: not a pseudo range
                }
            },
            Constellation::QZSS => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(20.0E-3_f64),
                            "C1C" => Some(1.0_f64), // TODO
                            "C1L" => Some(1.0_f64), // TODO
                            "C1X" => Some(1.0_f64), // TODO
                            "C1P" => Some(1.0_f64), // TODO,
                            "C1W" => Some(1.0_f64), // TODO
                            "C1Y" => Some(1.0_f64), // TODO
                            "C1M" => Some(1.0_f64), // TODO
                            "C2" => Some(1.0_f64),  //TODO
                            "C2S" => Some(1.0_f64), //TODO
                            "C2L" => Some(1.0_f64), //TODO
                            "C2X" => Some(1.0_f64), //TODO
                            "C5" => Some(1.0_f64),  //TODO
                            "C5I" => Some(1.0_f64), //TODO
                            "C5P" => Some(1.0_f64), //TODO
                            "C5Q" => Some(1.0_f64), //TODO
                            "C5X" => Some(1.0_f64), //TODO
                            "C5Z" => Some(1.0_f64), //TODO
                            "C6" => Some(1.0_f64),  //TODO
                            "C6L" => Some(1.0_f64), //TODO
                            "C6X" => Some(1.0_f64), //TODO
                            "C6E" => Some(1.0_f64), //TODO
                            "C6S" => Some(1.0_f64), //TODO
                            _ => None,              // does not apply
                        }
                    },
                    _ => None, // invalid: not a pseudo range
                }
            },
            Constellation::BeiDou => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(1.0_f64),
                            "C2I" => Some(1.0_f64),
                            "C2X" => Some(1.0_f64),
                            "C1D" => Some(1.0_f64),
                            "C1P" => Some(1.0_f64),
                            "C1X" => Some(1.0_f64),
                            "C1S" => Some(1.0_f64),
                            "C1L" => Some(1.0_f64),
                            "C1Z" => Some(1.0_f64),
                            "C5D" => Some(1.0_f64),
                            "C5P" => Some(1.0_f64),
                            "C5X" => Some(1.0_f64),
                            "C2" => Some(1.0_f64),
                            "C7I" => Some(1.0_f64),
                            "C7Q" => Some(1.0_f64),
                            "C7X" => Some(1.0_f64),
                            "C7D" => Some(1.0_f64),
                            "C7P" => Some(1.0_f64),
                            "C7Z" => Some(1.0_f64),
                            "C8D" => Some(1.0_f64),
                            "C8P" => Some(1.0_f64),
                            "C8X" => Some(1.0_f64),
                            "C6I" => Some(1.0_f64),
                            "C6Q" => Some(1.0_f64),
                            "C6X" => Some(1.0_f64),
                            "C6D" => Some(1.0_f64),
                            "C6P" => Some(1.0_f64),
                            "C6Z" => Some(1.0_f64),
                            _ => None, // does not apply
                        }
                    },
                    _ => None, // invalid : not a pseudo range
                }
            },
            Constellation::Galileo => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(1.0_f64),  // TODO
                            "C1A" => Some(1.0_f64), // TODO
                            "C1B" => Some(1.0_f64), // TODO
                            "C1C" => Some(1.0_f64), // TODO
                            "C1X" => Some(1.0_f64), // TODO
                            "C1Z" => Some(1.0_f64), // TODO
                            "C5I" => Some(1.0_f64), // TODO
                            "C5Q" => Some(1.0_f64), // TODO
                            "C5X" => Some(1.0_f64), // TODO
                            "C7I" => Some(1.0_f64), // TODO
                            "C7Q" => Some(1.0_f64), // TODO
                            "C7X" => Some(1.0_f64), // TODO
                            "C5" => Some(1.0_f64),  // TODO
                            "C8I" => Some(1.0_f64), // TODO
                            "C8Q" => Some(1.0_f64), // TODO
                            "C8X" => Some(1.0_f64), // TODO
                            "C6" => Some(1.0_f64),  // TODO
                            "C6A" => Some(1.0_f64), // TODO
                            "C6B" => Some(1.0_f64), // TODO
                            "C6C" => Some(1.0_f64), // TODO
                            "C6X" => Some(1.0_f64), // TODO
                            "C6Z" => Some(1.0_f64), // TODO
                            _ => None,
                        }
                    },
                    _ => None, // invalid: not a pseudo range
                }
            },
            Constellation::SBAS => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(1.0_f64),  // TODO
                            "C1C" => Some(1.0_f64), // TODO
                            "C5" => Some(1.0_f64),  // TODO
                            "C5I" => Some(1.0_f64), // TODO
                            "C5Q" => Some(1.0_f64), // TODO
                            "C5X" => Some(1.0_f64), // TODO
                            _ => None,
                        }
                    },
                    _ => None, // invalid: not a pseudo range
                }
            },
            Constellation::Glonass => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "C1" => Some(1.0_f64),  // TODO
                            "C1C" => Some(1.0_f64), // TODO
                            "C1P" => Some(1.0_f64), // TODO
                            "C4A" => Some(1.0_f64), // TODO
                            "C4C" => Some(1.0_f64), // TODO
                            "C5I" => Some(1.0_f64), // TODO
                            "C5Q" => Some(1.0_f64), // TODO
                            "C5X" => Some(1.0_f64), // TODO
                            _ => None,
                        }
                    },
                    _ => None, // invalid: not a pseudo range
                }
            },
            Constellation::IRNSS => {
                match self {
                    Self::PseudoRange(code) => {
                        match code.as_ref() {
                            "S" => Some(1.0_f64), //TODO
                            _ => None,            // invalid
                        }
                    },
                    _ => None, // invalid : not a pseudo range
                }
            },
            _ => None,
        }
    }
}

impl std::fmt::Display for Observable {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Pressure => write!(f, "PR"),
            Self::Temperature => write!(f, "TD"),
            Self::HumidityRate => write!(f, "HR"),
            Self::ZenithWetDelay => write!(f, "ZW"),
            Self::ZenithDryDelay => write!(f, "ZD"),
            Self::ZenithTotalDelay => write!(f, "ZT"),
            Self::WindDirection => write!(f, "WD"),
            Self::WindSpeed => write!(f, "WS"),
            Self::RainIncrement => write!(f, "RI"),
            Self::HailIndicator => write!(f, "HI"),
            Self::SSI(c) | Self::Phase(c) | Self::Doppler(c) | Self::PseudoRange(c) => {
                write!(f, "{}", c)
            },
        }
    }
}

impl std::str::FromStr for Observable {
    type Err = ParsingError;
    fn from_str(content: &str) -> Result<Self, Self::Err> {
        let content = content.to_uppercase();
        let content = content.trim();
        match content {
            "PR" => Ok(Self::Pressure),
            "TD" => Ok(Self::Temperature),
            "HR" => Ok(Self::HumidityRate),
            "ZW" => Ok(Self::ZenithWetDelay),
            "ZD" => Ok(Self::ZenithDryDelay),
            "ZT" => Ok(Self::ZenithTotalDelay),
            "WD" => Ok(Self::WindDirection),
            "WS" => Ok(Self::WindSpeed),
            "RI" => Ok(Self::RainIncrement),
            "HI" => Ok(Self::HailIndicator),
            _ => {
                let len = content.len();
                if len > 1 && len < 4 {
                    if content.starts_with("L") {
                        Ok(Self::Phase(content.to_string()))
                    } else if content.starts_with("C") || content.starts_with("P") {
                        Ok(Self::PseudoRange(content.to_string()))
                    } else if content.starts_with("S") {
                        Ok(Self::SSI(content.to_string()))
                    } else if content.starts_with("D") {
                        Ok(Self::Doppler(content.to_string()))
                    } else {
                        Err(ParsingError::UnknownObservable(content.to_string()))
                    }
                } else {
                    Err(ParsingError::MalformedDescriptor(content.to_string()))
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
    fn test_default_observable() {
        let default = Observable::default();
        assert_eq!(default, Observable::from_str("L1C").unwrap());
        assert_eq!(default, Observable::Phase(String::from("L1C")));
        assert!(default.is_phase_observable());
    }
    #[test]
    fn test_physics() {
        assert!(Observable::from_str("L1").unwrap().is_phase_observable());
        assert!(Observable::from_str("L2").unwrap().is_phase_observable());
        assert!(Observable::from_str("L6X").unwrap().is_phase_observable());
        assert!(Observable::from_str("C1")
            .unwrap()
            .is_pseudorange_observable());
        assert!(Observable::from_str("C2")
            .unwrap()
            .is_pseudorange_observable());
        assert!(Observable::from_str("C6X")
            .unwrap()
            .is_pseudorange_observable());
        assert!(Observable::from_str("D1").unwrap().is_doppler_observable());
        assert!(Observable::from_str("D2").unwrap().is_doppler_observable());
        assert!(Observable::from_str("D6X").unwrap().is_doppler_observable());
        assert!(Observable::from_str("S1").unwrap().is_ssi_observable());
        assert!(Observable::from_str("S2").unwrap().is_ssi_observable());
        assert!(Observable::from_str("S1P").unwrap().is_ssi_observable());
        assert!(Observable::from_str("S1W").unwrap().is_ssi_observable());
    }
    #[test]
    fn test_observable() {
        let obs = Observable::from_str("PR");
        assert_eq!(obs, Ok(Observable::Pressure));
        assert_eq!(obs.clone().unwrap().to_string(), "PR");
        assert_eq!(Observable::from_str("pr"), obs.clone());

        let obs = Observable::from_str("WS");
        assert_eq!(obs, Ok(Observable::WindSpeed));
        assert_eq!(obs.clone().unwrap().to_string(), "WS");
        assert_eq!(Observable::from_str("ws"), obs.clone());

        let obs = Observable::from_str("Wa");
        assert!(obs.is_err());

        assert_eq!(
            Observable::from_str("L1"),
            Ok(Observable::Phase(String::from("L1")))
        );
        assert!(Observable::from_str("L1").unwrap().code().is_none());

        assert_eq!(
            Observable::from_str("L2"),
            Ok(Observable::Phase(String::from("L2")))
        );
        assert_eq!(
            Observable::from_str("L5"),
            Ok(Observable::Phase(String::from("L5")))
        );
        assert_eq!(
            Observable::from_str("L6Q"),
            Ok(Observable::Phase(String::from("L6Q")))
        );
        assert_eq!(
            Observable::from_str("L6Q").unwrap().code(),
            Some(String::from("6Q"))
        );

        assert_eq!(
            Observable::from_str("L1C"),
            Ok(Observable::Phase(String::from("L1C")))
        );
        assert_eq!(
            Observable::from_str("L1P"),
            Ok(Observable::Phase(String::from("L1P")))
        );
        assert_eq!(
            Observable::from_str("L8X"),
            Ok(Observable::Phase(String::from("L8X")))
        );

        assert_eq!(
            Observable::from_str("S7Q"),
            Ok(Observable::SSI(String::from("S7Q")))
        );
        assert_eq!(
            format!("{}", Observable::PseudoRange(String::from("S7Q"))),
            "S7Q"
        );

        assert_eq!(
            Observable::from_str("D7Q"),
            Ok(Observable::Doppler(String::from("D7Q")))
        );
        assert_eq!(
            format!("{}", Observable::Doppler(String::from("D7Q"))),
            "D7Q"
        );

        assert_eq!(
            Observable::from_str("C7X"),
            Ok(Observable::PseudoRange(String::from("C7X")))
        );
        assert_eq!(
            format!("{}", Observable::PseudoRange(String::from("C7X"))),
            "C7X"
        );
    }
}
