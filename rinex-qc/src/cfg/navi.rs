use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use crate::cfg::ConfigError;

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum QcFrameModel {
    #[default]
    ITRF93,
    IAU,
}

impl std::fmt::Display for QcFrameModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ITRF93 => write!(f, "ITRF93"),
            Self::IAU => write!(f, "IAU"),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub enum QcPreferedOrbit {
    #[cfg(feature = "sp3")]
    SP3,
    #[default]
    RINEX,
}

impl std::fmt::Display for QcPreferedOrbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sp3")]
            Self::SP3 => write!(f, "SP3"),
            Self::RINEX => write!(f, "RINEX"),
        }
    }
}

impl std::str::FromStr for QcPreferedOrbit {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.contains("rinex") {
            Ok(Self::RINEX)
        } else {
            #[cfg(feature = "sp3")]
            if trimmed.contains("sp3") {
                return Ok(Self::SP3);
            }
            Err(ConfigError::PreferedOrbit)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcNaviOpts {
    #[serde(default)]
    pub frame_model: QcFrameModel,
    #[serde(default)]
    pub prefered_orbits: QcPreferedOrbit,
}

impl Render for QcNaviOpts {
    fn render(&self) -> Markup {
        html! {
            tr {
                th {
                    "Frame Model"
                }
                td {
                    (self.frame_model.to_string())
                }
            }
            tr {
                th {
                    "Prefered Orbits"
                }
                td {
                    (self.prefered_orbits.to_string())
                }
            }
        }
    }
}
