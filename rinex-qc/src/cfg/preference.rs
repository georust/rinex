use crate::cfg::QcConfigError;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum QcPreferedRoversSorting {
    /// Sort per Receiver model (needs to be defined)
    #[default]
    #[serde(rename(serialize = "gnss-rx", deserialize = "gnss-rx"))]
    Receiver,
    /// Sort per Antenna model.
    /// Use this if your datasets describes the same receiver model
    /// and you operate several, with different antenna specs
    #[serde(rename(serialize = "rx-antenna", deserialize = "rx-antenna"))]
    Antenna,
    /// Sort per Geodetic marker name/number.
    /// This usually applies to static surveying only along data produced
    /// by professional agencies that own a well professionnaly calibrated geodetic marker.
    #[serde(rename(serialize = "geodetic", deserialize = "geodetic"))]
    Geodetic,
    /// Sort per Operator ("Observer" in RINEx)
    #[serde(rename(serialize = "operator", deserialize = "operator"))]
    Operator,
}

impl std::str::FromStr for QcPreferedRoversSorting {
    type Err = QcConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.eq("gnss-rx") {
            Ok(Self::Receiver)
        } else if trimmed.eq("rx-antenna") {
            Ok(Self::Antenna)
        } else if trimmed.eq("geodetic") {
            Ok(Self::Geodetic)
        } else if trimmed.eq("operator") {
            Ok(Self::Operator)
        } else {
            Err(QcConfigError::ObservationsSorting)
        }
    }
}

impl std::fmt::Display for QcPreferedRoversSorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Antenna => write!(f, "RX-Antenna"),
            Self::Receiver => write!(f, "GNSS-RX"),
            Self::Geodetic => write!(f, "Geodetic Marker"),
            Self::Operator => write!(f, "Operator"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub enum QcPreferedOrbit {
    Any,
    #[cfg(feature = "sp3")]
    SP3,
    #[default]
    RINEx,
}

impl std::str::FromStr for QcPreferedOrbit {
    type Err = QcConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        if s.eq("*") {
            Ok(Self::Any)
        } else if s.eq("rinex") {
            Ok(Self::RINEx)
        } else {
            if s.eq("sp3") {
                #[cfg(feature = "sp3")]
                return Ok(Self::SP3);
                #[cfg(not(feature = "sp3"))]
                return Err(QcConfigError::SP3NotSupported);
            } else {
                Err(QcConfigError::PreferedOrbit)
            }
        }
    }
}

impl std::fmt::Display for QcPreferedOrbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sp3")]
            Self::SP3 => write!(f, "SP3"),
            Self::RINEx => write!(f, "RINEx"),
            Self::Any => write!(f, "Any"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub enum QcPreferedClock {
    Any,
    #[cfg(feature = "sp3")]
    SP3,
    #[default]
    RINEx,
}

impl std::fmt::Display for QcPreferedClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sp3")]
            Self::SP3 => write!(f, "SP3"),
            Self::RINEx => write!(f, "RINEx"),
            Self::Any => write!(f, "Any"),
        }
    }
}

/// Preference settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcPreferedSettings {
    /// Prefered clock source used only in PPP scenarios
    pub clk_source: QcPreferedClock,
    /// Prefered orbital source used only in PPP scenarios
    pub orbit_source: QcPreferedOrbit,
    /// Prefered classification (indexing) method
    pub rovers_sorting: QcPreferedRoversSorting,
}
