use crate::{constellation, Constellation};
use std::str::FromStr;
use strum_macros::EnumString;
use thiserror::Error;

/// Reference System parsing error
#[derive(Error, Debug)]
pub enum Error {
    #[error("unknown gnss constellation")]
    ConstellationError(#[from] constellation::Error),
    #[error("unknown reference system")]
    UnknownRefSystem,
}

/// RefSystem "Reference System" describes either reference GNSS
/// constellation, from which TEC maps were evaluated,
/// or theoretical model used
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RefSystem {
    /// Reference Constellation.
    /// When `Mixed` this generally means GPS + Glonass.
    /// When GNSS constellation was used, TEC maps
    /// include electron content through the ionosphere
    /// and plasmasphere, up to altitude 20000 km.
    GnssConstellation(Constellation),
    /// Other observation systems
    ObservationSystem(ObsSystem),
    /// Theoretical Model.
    /// When a theoretical model is used, refer to
    /// the Description provided in [crate::ionex::HeaderFields]
    /// for further explanations
    Model(Model),
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ObsSystem {
    /// BENt
    #[strum(serialize = "BEN")]
    BENt,
    /// ENVisat is an ESA Earth Observation satellite
    #[default]
    #[strum(serialize = "ENV")]
    ENVisat,
    /// European Remote Sensing Satellite (ESA).
    /// ERS-1 or ERS-2 were Earth observation satellites.
    /// Now replaced by ENVisat.
    ERS,
    /// IRI: Earth Observation Application group
    IRI,
}

impl std::fmt::Display for ObsSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

#[derive(Default, Debug, Clone, PartialEq, PartialOrd, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Model {
    /// Mixed / combined models.
    #[default]
    MIX,
    /// NNS transit
    NNS,
    /// TOP means TOPex.
    /// TOPex/TEC represents the ionosphere electron content
    /// measured over sea surface at altitudes below
    /// satellite orbits (1336 km).
    TOP,
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl Default for RefSystem {
    fn default() -> Self {
        Self::GnssConstellation(Constellation::default())
    }
}

impl std::fmt::Display for RefSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::GnssConstellation(c) => c.fmt(f),
            Self::ObservationSystem(s) => s.fmt(f),
            Self::Model(m) => m.fmt(f),
        }
    }
}

impl FromStr for RefSystem {
    type Err = Error;
    fn from_str(system: &str) -> Result<Self, Self::Err> {
        if let Ok(gnss) = Constellation::from_str(system) {
            Ok(Self::GnssConstellation(gnss))
        } else {
            if system.eq("GNSS") {
                Ok(Self::GnssConstellation(Constellation::Mixed))
            } else {
                if let Ok(obs) = ObsSystem::from_str(system) {
                    Ok(Self::ObservationSystem(obs))
                } else {
                    if let Ok(m) = Model::from_str(system) {
                        Ok(Self::Model(m))
                    } else {
                        Err(Error::UnknownRefSystem)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_refsystem() {
        let default = RefSystem::default();
        assert_eq!(
            default,
            RefSystem::GnssConstellation(Constellation::default())
        );
    }
}
