use thiserror::Error;

use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub mod navi;
pub mod report;
pub mod rover;

pub use navi::{QcFrameModel, QcNaviOpts};
pub use report::{QcReportOpts, QcReportType};
pub use rover::QcCustomRoverOpts;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("invalid prefered orbital source")]
    PreferedOrbit,
    #[error("invalid report type")]
    ReportType,
    #[error("invalid observation sorting method")]
    ObservationsSorting,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum QcPreferedObsSorting {
    /// Sort Observation datasets per Receiver model (needs to be defined)
    #[default]
    #[serde(rename(serialize = "rcvr", deserialize = "rcvr"))]
    Receiver,
    /// Sort Observation datasets per Antenna model.
    /// Use this if your datasets describes the same receiver model
    /// and you operate several, with different antenna specs
    #[serde(rename(serialize = "antenna", deserialize = "antenna"))]
    Antenna,
    /// Sort observation datasets per Geodetic marker name/number.
    /// Prefer this when operating to/from GNSS agencies owing Geodetic markers
    #[serde(rename(serialize = "geodetic", deserialize = "geodetic"))]
    Geodetic,
}

impl std::str::FromStr for QcPreferedObsSorting {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_ascii_lowercase();
        if trimmed.eq("rcvr") {
            Ok(Self::Receiver)
        } else if trimmed.eq("antenna") {
            Ok(Self::Antenna)
        } else if trimmed.eq("geodetic") {
            Ok(Self::Geodetic)
        } else {
            Err(ConfigError::ObservationsSorting)
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcConfig {
    #[serde(default)]
    pub workspace: PathBuf,
    #[serde(default)]
    pub obs_sorting: QcPreferedObsSorting,
    #[serde(default)]
    pub report: QcReportOpts,
    #[serde(default)]
    pub navi: QcNaviOpts,
    #[serde(default)]
    pub rover: QcCustomRoverOpts,
}

impl QcConfig {
    pub fn with_workspace<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut s = self.clone();
        s.workspace = path.as_ref().to_path_buf();
        s
    }
}

impl Render for QcConfig {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Reporting"
                        }
                        td {
                            (self.report.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Navigation settings"
                        }
                        td {
                            (self.navi.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Rover settings"
                        }
                        td {
                            (self.rover.render())
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use serde_json;
    use std::{fs::File, io::Write};

    use super::QcConfig;

    #[test]
    fn default_json_config() {
        let cfg: QcConfig = QcConfig::default();
        let mut fd = File::create("default.json").unwrap();

        let content = serde_json::to_string_pretty(&cfg).unwrap();
        write!(fd, "{}", content).unwrap();
    }

    #[test]
    fn manual_reference_json_config() {
        let mut cfg = QcConfig::default();

        let mut fd = File::create("manual-ref.json").unwrap();

        let content = serde_json::to_string_pretty(&cfg).unwrap();
        write!(fd, "{}", content).unwrap();
    }
}
