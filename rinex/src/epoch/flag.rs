use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// `EpochFlag` validates an epoch, 
/// or describes possible events that occurred
#[derive(Copy, Clone, Debug)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EpochFlag {
    /// Epoch is sane
    Ok,
    /// Power failure since previous epoch
    PowerFailure,
    /// Antenna is being moved at current epoch
    AntennaBeingMoved,
    /// Site has changed, received has moved since last epoch
    NewSiteOccupation,
    /// New information to come after this epoch
    HeaderInformationFollows,
    /// External event - significant event in this epoch
    ExternalEvent,
    /// Cycle slip at this epoch
    CycleSlip,
}

impl Default for EpochFlag {
    fn default() -> EpochFlag { EpochFlag::Ok }
}

impl EpochFlag {
    /// Returns True if self is a valid epoch
    pub fn is_ok (self) -> bool { self == EpochFlag::Ok }
}

impl FromStr for EpochFlag {
    type Err = std::io::Error;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(EpochFlag::Ok),
            "1" => Ok(EpochFlag::PowerFailure),
            "2" => Ok(EpochFlag::AntennaBeingMoved),
            "3" => Ok(EpochFlag::NewSiteOccupation),
            "4" => Ok(EpochFlag::HeaderInformationFollows),
            "5" => Ok(EpochFlag::ExternalEvent),
            "6" => Ok(EpochFlag::CycleSlip),
            _ => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid epoch flag value")),
        }
    }
}

impl std::fmt::Display for EpochFlag {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            EpochFlag::Ok => f.write_str("0"),
            EpochFlag::PowerFailure => f.write_str("1"),
            EpochFlag::AntennaBeingMoved => f.write_str("2"),
            EpochFlag::NewSiteOccupation => f.write_str("3"),
            EpochFlag::HeaderInformationFollows => f.write_str("4"),
            EpochFlag::ExternalEvent => f.write_str("5"),
            EpochFlag::CycleSlip => f.write_str("6"),
        }
    }
}
