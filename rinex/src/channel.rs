//! Carrier channels and associated methods
use crate::constellation::Constellation;
use crate::sv;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Channel {
    /// L1 (GPS, SBAS, QZSS)
    L1,
    /// L2 (GPS, QZSS)
    L2,
    /// L5 (GPS, SBAS), QZSS
    L5,
    /// L6 (LEX) QZSS
    L6,
    /// Glonass channel 1 with possible channel offset
    G1(Option<u8>),
    /// Glonass channel 2 with possible channel offset
    G2(Option<u8>),
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
    /// B1C BeiDou 1C
    B1C,
    /// B1A BeiDou 1A
    B1A,
    /// B2: BeiDou 2
    B2,
    /// B3
    B3,
    /// IRNSS S
    S,
}

impl Default for Channel {
    fn default() -> Channel {
        Channel::L1
    }
}

#[derive(Error, Debug)]
pub enum Error {
    /// Unable to parse Channel from given string content
    #[error("unable to parse channel from content \"{0}\"")]
    ParseError(String),
    #[error("unable to identify glonass channel from \"{0}\"")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("unable to identify constellation + carrier code")]
    SvError(#[from] sv::Error),
    #[error("non recognized observable \"{0}\"")]
    InvalidObservable(String),
}

impl FromStr for Channel {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.contains("L1") {
            Ok(Channel::L1)
        } else if s.contains("L2") {
            Ok(Channel::L2)
        } else if s.contains("L5") {
            Ok(Channel::L5)
        } else if s.contains("G1") {
            if s.eq("G1") {
                Ok(Channel::G1(None))
            } else if s.contains("G1(") {
                let items: Vec<&str> = s.split("(").collect();
                let item = items[1].replace(")", "");
                Ok(Channel::G1(Some(u8::from_str_radix(&item, 10)?)))
            } else {
                Err(Error::ParseError(s.to_string()))
            }
        } else if s.contains("G2") {
            if s.eq("G2") {
                Ok(Channel::G2(None))
            } else if s.contains("G2(") {
                let items: Vec<&str> = s.split("(").collect();
                let item = items[1].replace(")", "");
                Ok(Channel::G2(Some(u8::from_str_radix(&item, 10)?)))
            } else {
                Err(Error::ParseError(s.to_string()))
            }
        } else {
            Err(Error::ParseError(s.to_string()))
        }
    }
}

impl Channel {
    /// Returns frequency associated to this channel in MHz
    pub fn carrier_frequency_mhz(&self) -> f64 {
        match self {
            Channel::L1 | Channel::E1 => 1575.42_f64,
            Channel::L2 | Channel::E2 => 1227.60_f64,
            Channel::L5 => 1176.45_f64,
            Channel::E5 => 1191.795_f64,
            Channel::E6 => 1278.75_f64,
            Channel::G1(Some(c)) => 1602.0_f64 + (*c as f64 * 9.0 / 16.0),
            Channel::G1(_) => 1602.0_f64,
            Channel::G2(Some(c)) => 1246.06_f64 + (*c as f64 * 7.0 / 16.0),
            Channel::G2(_) => 1246.06_f64,
            Channel::G3 => 1202.025_f64,
            Channel::L6 => 1278.75_f64,
            Channel::B1 => 1561.098_f64,
            Channel::B1A => 1575.42_f64,
            Channel::B1C => 1575.42_f64,
            Channel::B2 => 1207.140_f64,
            Channel::B3 => 1268.52_f64,
            Channel::S => 2492.028_f64,
        }
    }
    /// Returns wavelength of this channel
    pub fn carrier_wavelength(&self) -> f64 {
        299792458.0 / self.carrier_frequency_mhz() / 10.0E6
    }

    /// Returns channel bandwidth in MHz
    pub fn bandwidth_mhz(&self) -> f64 {
        match self {
            Channel::L1 | Channel::G1(_) | Channel::E1 => 15.345_f64,
            Channel::L2 | Channel::G2(_) | Channel::E2 => 11.0_f64,
            Channel::L5 | Channel::E5 => 12.5_f64,
            Channel::G3 => todo!("G3 bandwidth is not known to this day"),
            Channel::E6 => todo!("E6 bandwidth is not known to this day"),
            Channel::L6 => todo!("L6 bandwidth is not known to this day"),
            Channel::S => todo!("S bandwidth is not known to this day"),
            Channel::B1 => todo!("B1 bandwidth is not known to this day"),
            Channel::B1A => todo!("B1A bandwidth is not known to this day"),
            Channel::B1C => todo!("B1C bandwidth is not known to this day"),
            Channel::B2 => todo!("B2 bandwidth is not known to this day"),
            Channel::B3 => todo!("B3 bandwidth is not known to this day"),
        }
    }

    /// Identifies Frequency channel, from given observable, related
    /// to given Constellation
    pub fn from_observable(constellation: Constellation, observable: &str) -> Result<Self, Error> {
        match constellation {
            Constellation::GPS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("2") {
                    Ok(Self::L2)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::Glonass => {
                if observable.contains("1") {
                    Ok(Self::G1(None))
                } else if observable.contains("2") {
                    Ok(Self::G2(None))
                } else if observable.contains("3") {
                    Ok(Self::G3)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::Galileo => {
                if observable.contains("1") {
                    Ok(Self::E1)
                } else if observable.contains("2") {
                    Ok(Self::E2)
                } else if observable.contains("5") {
                    Ok(Self::E5)
                } else if observable.contains("6") {
                    Ok(Self::E6)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::SBAS(_) => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::QZSS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("2") {
                    Ok(Self::L2)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            Constellation::IRNSS => {
                if observable.contains("1") {
                    Ok(Self::L1)
                } else if observable.contains("5") {
                    Ok(Self::L5)
                } else {
                    Err(Error::InvalidObservable(observable.to_string()))
                }
            },
            _ => todo!(
                "not implemented for constellation \"{}\" yet..",
                constellation.to_3_letter_code()
            ),
        }
    }

    /// Builds a Channel Frequency from an `Sv` 3 letter code descriptor,
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
        assert!(Channel::from_str("L1").is_ok());
        assert!(Channel::from_str("C1").is_err());
        assert!(Channel::from_str("L5").is_ok());

        let l1 = Channel::from_str("L1").unwrap();
        assert_eq!(l1.carrier_frequency_mhz(), 1575.42_f64);
        assert_eq!(l1.carrier_wavelength(), 299792458.0 / 1575.42_f64 / 10.0E6);
        let channel = Channel::from_observable(Constellation::GPS, "L1C");
        assert!(channel.is_ok());
        let channel = channel.unwrap();
        assert_eq!(channel, Channel::L1);
    }
}
