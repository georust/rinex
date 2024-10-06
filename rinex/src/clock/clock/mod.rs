use crate::prelude::{
    DOMES,
    Version,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod profile;
pub use profile::{ClockProfile, ClockProfileType};

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

impl std::fmt::Display for ClockType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(sv) = self.as_sv() {
            f.write_str(&sv.to_string())?
        } else if let Some(station) = self.as_station() {
            f.write_str(&station)?
        }
        Ok(())
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

/// [Clock] definition
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Clock {
    /// Name of this local clock
    pub name: String,
    /// Clock site DOMES ID#
    pub domes: Option<DOMES>,
    /// Possible clock constraint [s]
    pub constraint: Option<f64>,
}

impl Clock {
    /// Parses [Clock] from passed content
    pub(crate) fn parse(version: Version, content: &str) -> Self {
        const LIMIT: Version = Version { major: 3, minor: 4 };
        if version < LIMIT {
            let (name, rem) = content.split_at(4);
            let (domes, rem) = rem.split_at(36);
            let constraint = rem.split_at(20).0;
            Self {
                name: name.trim().to_string(),
                domes: if let Ok(domes) = DOMES::from_str(domes.trim()) {
                    Some(domes)
                } else {
                    None
                },
                constraint: if let Ok(value) = constraint.trim().parse::<f64>() {
                    Some(value)
                } else {
                    None
                },
            }
        } else {
            let (name, rem) = content.split_at(10);
            let (domes, rem) = rem.split_at(10);
            let constraint = rem.split_at(40).0;
            Self {
                name: name.trim().to_string(),
                domes: if let Ok(domes) = DOMES::from_str(domes.trim()) {
                    Some(domes)
                } else {
                    None
                },
                constraint: if let Ok(value) = constraint.trim().parse::<f64>() {
                    Some(value)
                } else {
                    None
                },
            }
        }
    }
}
