//! NAV RINEX module
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
pub(crate) use ionmessage::parse_4_fields;
pub use ionmessage::{BdModel, IonMessage, KbModel, KbRegionCode, NgModel, NgRegionFlags};
pub use orbits::OrbitItem;
pub use record::{NavFrame, NavMsgType, Record};
pub use stomessage::StoMessage;

use crate::epoch;
use thiserror::Error;

use gnss::prelude::SV;

/// Navigation Record Parsing Error
#[derive(Error, Debug)]
pub enum Error {
    #[error("epoch is missing data")]
    MissingData,
    #[error("file operation error")]
    FileIoError(#[from] std::io::Error),
    #[error("failed to locate revision in db")]
    OrbitRevision,
    #[error("unknown nav frame class")]
    UnknownFrameClass,
    #[error("unknown nav message type")]
    UnknownNavMsgType,
    #[error("sv parsing error")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to parse orbit field")]
    ParseOrbitError(#[from] orbits::OrbitItemError),
    #[error("failed to parse sv::prn")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse sv clock fields")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] epoch::ParsingError),
    #[error("failed to identify class/type")]
    StrumError(#[from] strum::ParseError),
    #[error("failed to parse EPH message")]
    EphMessageError(#[from] ephemeris::Error),
    #[error("failed to parse ION message")]
    IonMessageError(#[from] ionmessage::Error),
    #[error("failed to parse EOP message")]
    EopMessageError(#[from] eopmessage::Error),
    #[error("failed to parse STO message")]
    StoMessageError(#[from] stomessage::Error),
    #[error("failed to identify timescale for {0}")]
    TimescaleIdentification(SV),
}

/*
 * Marker to identify which NAV frame follows in the record
 */
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
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c = s.to_uppercase();
        let c = c.trim();
        match c {
            "EPH" => Ok(Self::Ephemeris),
            "STO" => Ok(Self::SystemTimeOffset),
            "EOP" => Ok(Self::EarthOrientation),
            "ION" => Ok(Self::IonosphericModel),
            _ => Err(Error::UnknownFrameClass),
        }
    }
}
