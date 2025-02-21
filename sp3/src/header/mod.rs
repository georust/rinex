//! header parsing utilities
pub(crate) mod line1;
pub(crate) mod line2;

pub mod version;

use crate::{
    header::version::Version,
    prelude::{Constellation, Duration, ParsingError, TimeScale, SV},
};

#[cfg(docsrs)]
use crate::prelude::Epoch;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    #[default]
    Position,
    Velocity,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Position => f.write_str("P"),
            Self::Velocity => f.write_str("V"),
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else if s.eq("V") {
            Ok(Self::Velocity)
        } else {
            Err(ParsingError::UnknownDataType)
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    BHN,
    HLM,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FIT => f.write_str("FIT"),
            Self::EXT => f.write_str("EXT"),
            Self::BCT => f.write_str("BCT"),
            Self::BHN => f.write_str("BHN"),
            Self::HLM => f.write_str("HLM"),
        }
    }
}

impl std::str::FromStr for OrbitType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FIT") {
            Ok(Self::FIT)
        } else if s.eq("EXT") {
            Ok(Self::EXT)
        } else if s.eq("BCT") {
            Ok(Self::BCT)
        } else if s.eq("BHN") {
            Ok(Self::BHN)
        } else if s.eq("HLM") {
            Ok(Self::HLM)
        } else {
            Err(ParsingError::UnknownOrbitType)
        }
    }
}

#[derive(Debug, Default, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    /// File revision as [Version]
    pub version: Version,
    /// [DataType] used in this file.
    /// [DataType::Velocity] means velocity vector will be provided in following record.
    pub data_type: DataType,
    /// Coordinates system description.
    pub coord_system: String,
    /// [OrbitType] used in the fitting process prior publication.
    pub orbit_type: OrbitType,
    /// Agency providing this record.
    pub agency: String,
    /// Type of [Constellation] found in this record.
    /// For example [Constellation::GPS] means you will only find GPS satellite vehicles.
    pub constellation: Constellation,
    /// [TimeScale] that applies to all following [Epoch]s.
    pub timescale: TimeScale,
    /// [TimeScale] week counter.
    pub week_counter: u32,
    /// [TimeScale] seconds in current week.
    pub week_sow: f64,
    /// Datetime of first record entry, expressed as integral and frational MJD in [TimeScale].
    pub mjd: f64,
    /// Sampling period, as [Duration].
    pub epoch_interval: Duration,
    /// [SV] to be found in this record.
    pub satellites: Vec<SV>,
}
