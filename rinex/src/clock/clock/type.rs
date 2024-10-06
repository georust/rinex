#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Type of Clock
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ClockType {
    /// [SV] onboard Clock
    SV(SV),
    /// Ground station Clock
    Station(String),
}

impl Default for ClockType {
    fn default() -> Self {
        Self::Station("Unknown".to_string())
    }
}

impl ClockType {
    /// [SV] onboard clock unwrapping attempt
    pub fn as_sv(&self) -> Option<SV> {
        match self {
            Self::SV(s) => Some(*s),
            _ => None,
        }
    }
    /// Station clock unwrapping attempt
    pub fn as_station(&self) -> Option<String> {
        match self {
            Self::Station(s) => Some(s.clone()),
            _ => None,
        }
    }
}