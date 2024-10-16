//! Navigation RINEX module

mod eopmessage;
mod ephemeris;
mod health;
mod ionmessage;
mod stomessage;

pub mod orbits;
pub mod record;

pub use eopmessage::EopMessage;
pub use ephemeris::Ephemeris;
pub use health::{GeoHealth, GloHealth, Health, IrnssHealth};
pub use ionmessage::{BdModel, IonMessage, KbModel, KbRegionCode, NgModel, NgRegionFlags};
pub use orbits::OrbitItem;
pub use record::{NavFrame, NavMsgType, Record};
pub use stomessage::StoMessage;

use crate::prelude::ParsingError;

/// Navigation Frame classes
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum FrameClass {
    #[default]
    Ephemeris,
    SystemTimeOffset,
    EarthOrientation,
    IonosphericModel,
}

impl std::fmt::Display for FrameClass {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ephemeris => f.write_str("EPH"),
            Self::SystemTimeOffset => f.write_str("STO"),
            Self::EarthOrientation => f.write_str("EOP"),
            Self::IonosphericModel => f.write_str("ION"),
        }
    }
}

impl std::str::FromStr for FrameClass {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "EPH" => Ok(Self::Ephemeris),
            "STO" => Ok(Self::SystemTimeOffset),
            "EOP" => Ok(Self::EarthOrientation),
            "ION" => Ok(Self::IonosphericModel),
            _ => Err(ParsingError::NavFrameClass),
        }
    }
}
