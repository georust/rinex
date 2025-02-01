use serde::{Deserialize, Serialize};

use crate::cfg::QcConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QcPreferedRover {
    Any,
    Prefered(String),
}

impl Default for QcPreferedRover {
    fn default() -> Self {
        Self::Any
    }
}

impl std::str::FromStr for QcPreferedRover {
    type Err = QcConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq("*") {
            Ok(Self::Any)
        } else {
            Ok(Self::Prefered(trimmed.to_string()))
        }
    }
}

impl std::fmt::Display for QcPreferedRover {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "*"),
            Self::Prefered(rover) => write!(f, "{}", rover),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcCustomRoverOpts {
    /// Manual RX position expressed as ECEF coordinates in km.
    pub manual_rx_ecef_km: Option<(f64, f64, f64)>,
    /// Prefered rover, for which we will solve solutions
    pub prefered_rover: QcPreferedRover,
}
