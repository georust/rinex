use crate::{navigation::Ephemeris, prelude::ParsingError};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Navigation Frame classes
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NavFrameType {
    /// Ephemeris frame
    #[default]
    Ephemeris,
    /// System Time Offset frame
    SystemTimeOffset,
    /// Earth Orientation frame
    EarthOrientation,
    /// Ionosphere model
    IonosphereModel,
}

impl std::fmt::Display for NavFrameType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphereModel => f.write_str("ION"),
        }
    }
}

impl std::str::FromStr for NavFrameType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "EPH" => Ok(Self::Ephemeris),
            "STO" => Ok(Self::SystemTimeOffset),
            "EOP" => Ok(Self::EarthOrientation),
            "ION" => Ok(Self::IonosphereModel),
            _ => Err(ParsingError::NavFrameClass),
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum NavFrame {
    EPH(Ephemeris),
    // EOP(EopFrame),
    // ION(IonosphereModel),
    // STO(SystemTimeFrame),
}

impl NavFrame {
    /// [Ephemeris] unwrapping attempt.
    pub fn as_eph(&self) -> Option<&Ephemeris> {
        match self {
            Self::EPH(eph) => Some(eph),
            _ => None,
        }
    }

    /// Mutable [Ephemeris] unwrapping attempt.
    pub fn as_mut_eph(&mut self) -> Option<&mut Ephemeris> {
        match self {
            Self::EPH(eph) => Some(eph),
            _ => None,
        }
    }

    // pub fn as_ion(&self) -> Option<(NavMsgType, SV, &IonMessage)> {
    //     match self {
    //         Self::Ion(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }

    // pub fn as_mut_ion(&mut self) -> Option<(NavMsgType, SV, &mut IonMessage)> {
    //     match self {
    //         Self::Ion(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }

    // pub fn as_eop(&self) -> Option<(NavMsgType, SV, &EopMessage)> {
    //     match self {
    //         Self::Eop(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }

    // pub fn as_mut_eop(&mut self) -> Option<(NavMsgType, SV, &mut EopMessage)> {
    //     match self {
    //         Self::Eop(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }

    // pub fn as_sto(&self) -> Option<(NavMsgType, SV, &StoMessage)> {
    //     match self {
    //         Self::Sto(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }

    // pub fn as_mut_sto(&mut self) -> Option<(NavMsgType, SV, &mut StoMessage)> {
    //     match self {
    //         Self::Sto(msg, sv, fr) => Some((*msg, *sv, fr)),
    //         _ => None,
    //     }
    // }
}
