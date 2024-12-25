use thiserror::Error;

use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub mod navi;
pub mod preference;
pub mod report;
pub mod rover;

pub use navi::{QcFrameModel, QcNaviOpts};
pub use preference::QcPreferedSettings;
pub use report::{QcReportOpts, QcReportType};
pub use rover::QcCustomRoverOpts;

#[derive(Error, Debug)]
pub enum QcConfigError {
    #[error("invalid prefered orbital source")]
    PreferedOrbit,
    #[error("invalid report type")]
    ReportType,
    #[error("invalid observation sorting method")]
    ObservationsSorting,
    #[error("library built without sp3 support")]
    SP3NotSupported,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcConfig {
    #[serde(default)]
    pub workspace: PathBuf,
    #[serde(default)]
    pub preference: QcPreferedSettings,
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
                            "Preference"
                        }
                        td {
                            (self.preference.render())
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
