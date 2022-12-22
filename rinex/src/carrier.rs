//! Carrier channels and associated methods
use crate::constellation::Constellation;
use crate::sv;
use std::str::FromStr;
use thiserror::Error;

lazy_static! {
    pub(crate) static ref KNOWN_CODES: Vec<&'static str> = vec![
        "1A", "1B", "1C", "1D", "1L", "1M", "1P", "1S", "1W", "1X", "1Z", "2C", "2D", "2L", "2M",
        "2P", "2S", "2W", "3I", "3X", "3Q", "4A", "4B", "4X", "5A", "5B", "5C", "5I", "5P", "5Q",
        "5X", "6A", "6B", "6C", "6Q", "6X", "6Z", "7D", "7I", "7P", "7Q", "7X", "8D", "8P", "8I",
        "8Q", "8X", "9A", "9B", "9C", "9X",
    ];
}

//pub(crate) fn parse_glonass_channels(content: &str)

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Carrier {
    /// L1 (GPS, SBAS, QZSS)
    L1,
    /// L2 (GPS, QZSS)
    L2,
    /// L5 (GPS, SBAS), QZSS
    L5,
    /// L6 (LEX) QZSS
    L6,
    /// Glonass channel 1 with possible offset
    G1(Option<i8>),
    /// Glonass channel 2 with possible offset
    G2(Option<i8>),
    /// Glonass channel 3
    G3,
    /// E1: GAL
    E1,
    /// E2: GAL
    E2,
    /// E5: GAL E5a + E5b
    E5,
    /// E6: GAL military
    E6,
    /// B1: BeiDou 1
    B1,
    /// B1A BeiDou 1A
    B1A,
    /// B1C BeiDou 1C
    B1C,
    /// B2: BeiDou 2
    B2,
    /// B3
    B3,
    /// IRNSS S
    S,
}

impl Default for Carrier {
    fn default() -> Carrier {
        Carrier::L1
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    /// Unable to parse Carrier from given string content
    #[error("unable to parse channel from content \"{0}\"")]
    ParseError(String),
    #[error("unable to identify glonass channel from \"{0}\"")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unable to identify constellation + carrier code")]
    SvError(#[from] sv::Error),
    #[error("non recognized observable \"{0}\"")]
    InvalidObservable(String),
}

impl std::fmt::Display for Carrier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::L1 => write!(f, "L1"),
            Self::L2 => write!(f, "L2"),
            Self::L5 => write!(f, "L5"),
            Self::L6 => write!(f, "L6"),
            Self::G1(_) => write!(f, "L1"),
            Self::G2(_) => write!(f, "L2"),
            Self::G3 => write!(f, "L3"),
            Self::E1 => write!(f, "E1"),
            Self::E2 => write!(f, "E2"),
            Self::E5 => write!(f, "E5"),
            Self::E6 => write!(f, "E6"),
            Self::B1 => write!(f, "B1"),
            Self::B1A => write!(f, "B1A"),
            Self::B1C => write!(f, "B1C"),
            Self::B2 => write!(f, "B2"),
            Self::B3 => write!(f, "B3"),
            Self::S => write!(f, "S"),
        }
    }
}

impl std::str::FromStr for Carrier {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let content = s.to_uppercase();
        let content = content.trim();
        if content.eq("L1") {
            Ok(Carrier::L1)
        } else if content.eq("L2") {
            Ok(Carrier::L2)
        } else if content.eq("L3") {
            Ok(Carrier::G3)
        } else if content.eq("L5") {
            Ok(Carrier::L5)
        } else if content.eq("L6") {
            Ok(Carrier::L6)
        } else {
            Err(Error::ParseError(s.to_string()))
        }
    }
}

impl Carrier {
    /// Returns frequency associated to this channel in MHz
    pub fn carrier_frequency(&self) -> f64 {
        self.carrier_frequency_mhz() * 1.0E6
    }
    pub fn carrier_frequency_mhz(&self) -> f64 {
        match self {
            Carrier::L1 | Carrier::E1 => 1575.42_f64,
            Carrier::L2 | Carrier::E2 => 1227.60_f64,
            Carrier::L5 => 1176.45_f64,
            Carrier::E5 => 1191.795_f64,
            Carrier::E6 => 1278.75_f64,
            Carrier::G1(Some(c)) => 1602.0_f64 + (*c as f64 * 9.0 / 16.0),
            Carrier::G1(_) => 1602.0_f64,
            Carrier::G2(Some(c)) => 1246.06_f64 + (*c as f64 * 7.0 / 16.0),
            Carrier::G2(_) => 1246.06_f64,
            Carrier::G3 => 1207.140_f64,
            Carrier::L6 => 1278.75_f64,
            Carrier::B1 => 1561.098_f64,
            Carrier::B1A => 1575.42_f64,
            Carrier::B1C => 1575.42_f64,
            Carrier::B2 => 1207.140_f64,
            Carrier::B3 => 1268.52_f64,
            Carrier::S => 2492.028_f64,
        }
    }
    /// Returns wavelength of this channel
    pub fn carrier_wavelength(&self) -> f64 {
        299_792_458.0_f64 / self.carrier_frequency()
    }

    /// Returns channel bandwidth in MHz
    pub fn bandwidth_mhz(&self) -> f64 {
        match self {
            Carrier::L1 | Carrier::G1(_) | Carrier::E1 => 15.345_f64,
            Carrier::L2 | Carrier::G2(_) | Carrier::E2 => 11.0_f64,
            Carrier::L5 | Carrier::E5 => 12.5_f64,
            Carrier::G3 => todo!("G3 bandwidth is not known to this day"),
            Carrier::E6 => todo!("E6 bandwidth is not known to this day"),
            Carrier::L6 => todo!("L6 bandwidth is not known to this day"),
            Carrier::S => todo!("S bandwidth is not known to this day"),
            Carrier::B1 => todo!("B1 bandwidth is not known to this day"),
            Carrier::B1A => todo!("B1A bandwidth is not known to this day"),
            Carrier::B1C => todo!("B1C bandwidth is not known to this day"),
            Carrier::B2 => todo!("B2 bandwidth is not known to this day"),
            Carrier::B3 => todo!("B3 bandwidth is not known to this day"),
        }
    }

    /// Converts to exact Glonass carrier
    pub fn with_offset(&self, offset: i8) -> Self {
        match self {
            Self::L1 => Self::G1(Some(offset)),
            Self::L2 => Self::G2(Some(offset)),
            other => *other,
        }
    }

    /// Identifies Frequency channel, from given observable, related
    /// to given Constellation
    pub fn from_code(constellation: Constellation, code: &str) -> Result<Self, Error> {
        match constellation {
            Constellation::GPS => {
                if code.contains("1") {
                    Ok(Self::L1)
                } else if code.contains("2") {
                    Ok(Self::L2)
                } else if code.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            Constellation::Glonass => {
                if code.contains("1") {
                    Ok(Self::G1(None))
                } else if code.contains("2") {
                    Ok(Self::G2(None))
                } else if code.contains("3") {
                    Ok(Self::G3)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            Constellation::Galileo => {
                if code.contains("1") {
                    Ok(Self::E1)
                } else if code.contains("2") {
                    Ok(Self::E2)
                } else if code.contains("5") {
                    Ok(Self::E5)
                } else if code.contains("6") {
                    Ok(Self::E6)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            Constellation::SBAS(_) => {
                if code.contains("1") {
                    Ok(Self::L1)
                } else if code.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            Constellation::QZSS => {
                if code.contains("1") {
                    Ok(Self::L1)
                } else if code.contains("2") {
                    Ok(Self::L2)
                } else if code.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            Constellation::IRNSS => {
                if code.contains("1") {
                    Ok(Self::L1)
                } else if code.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(code.to_string()))
                }
            },
            _ => todo!("for \"{}\" consellation", constellation.to_3_letter_code()),
        }
    }

    /// Builds a Carrier Frequency from an `Sv` 3 letter code descriptor,
    /// mainly used in `ATX` RINEX for so called `frequency` field
    pub fn from_sv_code(code: &str) -> Result<Self, Error> {
        let sv = sv::Sv::from_str(code)?;
        match sv.constellation {
            Constellation::GPS => match sv.prn {
                1 => Ok(Self::L1),
                2 => Ok(Self::L2),
                5 => Ok(Self::L5),
                _ => Ok(Self::L1),
            },
            Constellation::Glonass => match sv.prn {
                1 => Ok(Self::G1(None)),
                2 => Ok(Self::G2(None)),
                _ => Ok(Self::G1(None)),
            },
            Constellation::Galileo => match sv.prn {
                1 => Ok(Self::E1),
                2 => Ok(Self::E2),
                5 => Ok(Self::E5),
                _ => Ok(Self::E1),
            },
            Constellation::SBAS(_) | Constellation::Geo => match sv.prn {
                1 => Ok(Self::L1),
                5 => Ok(Self::L5),
                _ => Ok(Self::L1),
            },
            Constellation::BeiDou => match sv.prn {
                1 => Ok(Self::E1),
                2 => Ok(Self::E2),
                5 => Ok(Self::E5),
                6 => Ok(Self::E6),
                _ => Ok(Self::E1),
            },
            Constellation::QZSS => match sv.prn {
                1 => Ok(Self::L1),
                2 => Ok(Self::L2),
                5 => Ok(Self::L5),
                _ => Ok(Self::L1),
            },
            Constellation::IRNSS => {
                match sv.prn {
                    // TODO: confirm!
                    1 => Ok(Self::L1),
                    5 => Ok(Self::L5),
                    _ => Ok(Self::L1),
                }
            },
            _ => panic!(
                "non supported conversion from {}",
                sv.constellation.to_3_letter_code()
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_channel() {
        assert!(Carrier::from_str("L1").is_ok());
        assert!(Carrier::from_str("C1").is_err());
        assert!(Carrier::from_str("L5").is_ok());

        let l1 = Carrier::from_str("L1").unwrap();
        assert_eq!(l1.carrier_frequency_mhz(), 1575.42_f64);
        assert_eq!(l1.carrier_wavelength(), 299792458.0 / 1_575_420_000.0_f64);

        let channel = Carrier::from_code(Constellation::GPS, "L1C");
        assert!(channel.is_ok());

        let channel = channel.unwrap();
        assert_eq!(channel, Carrier::L1);
    }
}
