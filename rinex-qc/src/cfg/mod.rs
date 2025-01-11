use thiserror::Error;

use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use std::path::{Path, PathBuf};

pub mod navi;
pub mod preference;
pub mod report;
pub mod rover;
pub mod solutions;

pub use navi::{QcFrameModel, QcNaviOpts};
pub use preference::{
    QcPreferedClock, QcPreferedOrbit, QcPreferedRoversSorting, QcPreferedSettings,
};
pub use report::{QcReportOpts, QcReportType};
pub use rover::{QcCustomRoverOpts, QcPreferedRover};
pub use solutions::QcSolutions;

#[cfg(feature = "nav")]
use gnss_rtk::prelude::Config as RTKConfig;

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
    /// Custom workspace location.
    /// This is where any report may be generated.
    #[serde(default)]
    pub workspace: PathBuf,

    /// Report rendition preferences.
    #[serde(default)]
    pub preference: QcPreferedSettings,

    /// Report rendition preferences.
    #[serde(default)]
    pub report: QcReportOpts,

    /// Navigation preferences.    
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    #[serde(default)]
    pub navi: QcNaviOpts,

    /// Custom "rover" preferences that may serve
    /// more than just navigation.
    #[serde(default)]
    pub rover: QcCustomRoverOpts,

    /// Custom navigation solver options.
    /// Used in post processed navigations, mostly when
    /// auto-integrating navigation solutions to analysis reports.
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    pub rtk_config: Option<RTKConfig>,

    /// Report synthesis will automatically attach
    /// the following solutions. By default, we do no attach any.
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    #[serde(default)]
    pub solutions: QcSolutions,
}

impl QcConfig {
    pub fn with_preferences(&self, preferences: QcPreferedSettings) -> Self {
        let mut s = self.clone();
        s.preference = preferences;
        s
    }

    pub fn with_rover_settings(&self, preferences: QcCustomRoverOpts) -> Self {
        let mut s = self.clone();
        s.rover = preferences;
        s
    }

    pub fn with_report_preferences(&self, preferences: QcReportOpts) -> Self {
        let mut s = self.clone();
        s.report = preferences;
        s
    }

    pub fn with_solutions(&self, solutions: QcSolutions) -> Self {
        let mut s = self.clone();
        s.solutions = solutions;
        s
    }

    /// Creates a new [QcConfig] with custom workspace location.
    pub fn with_workspace<P: AsRef<Path>>(&self, path: P) -> Self {
        let mut s = self.clone();
        s.workspace = path.as_ref().to_path_buf();
        s
    }

    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    pub fn with_navi_settings(&self, navi: QcNaviOpts) -> Self {
        let mut s = self.clone();
        s.navi = navi;
        s
    }

    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    pub fn with_rtk_config(&self, cfg: RTKConfig) -> Self {
        let mut s = self.clone();
        s.rtk_config = Some(cfg);
        s
    }

    /// Returns internal [RTKConfig] in any case
    #[cfg(feature = "nav")]
    pub(crate) fn rtk_config(&self) -> RTKConfig {
        if let Some(rtk_config) = &self.rtk_config {
            rtk_config.clone()
        } else {
            RTKConfig::default()
        }
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
                    @ if cfg!(feature = "nav") {
                        tr {
                            th class="is-info" {
                                "Solutions"
                            }
                            td {
                                (self.solutions.render())
                            }
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
