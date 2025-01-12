use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum QcFrameModel {
    #[default]
    ITRF93,
    IAU,
}

impl std::fmt::Display for QcFrameModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IAU => write!(f, "IAU"),
            Self::ITRF93 => write!(f, "ITRF93"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcNaviOpts {
    #[serde(default)]
    pub frame_model: QcFrameModel,
}
