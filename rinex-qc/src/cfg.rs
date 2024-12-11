use maud::{html, Markup, Render};
use rinex::prelude::*;
use thiserror::Error;

use serde::{Deserialize, Serialize};

/// Configuration Error
#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("invalid report type")]
    InvalidReportType,
}

use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Default, Deserialize)]
pub enum PreferedOrbit {
    #[cfg(feature = "sp3")]
    SP3,
    #[default]
    BrdcRadio,
}

impl std::fmt::Display for PreferedOrbit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "sp3")]
            Self::SP3 => write!(f, "SP3"),
            Self::BrdcRadio => write!(f, "Radio Broadcast"),
        }
    }
}

/// [QcReportType]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QcReportType {
    /// In [QcReportType::Summary] mode, only the summary section
    /// of the report is to be generated. It is the lightest
    /// form we can generate.
    Summary,
    /// In [QcReportType::Full] mode, all information is generated.
    #[default]
    Full,
}

impl FromStr for QcReportType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "sum" | "summ" | "summary" => Ok(Self::Summary),
            "full" => Ok(Self::Full),
            _ => Err(Error::InvalidReportType),
        }
    }
}

impl Display for QcReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Full => f.write_str("Full"),
            Self::Summary => f.write_str("Summary"),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct QcConfig {
    #[serde(default)]
    pub report: QcReportType,
    #[serde(default)]
    pub prefered_orbit: PreferedOrbit,
    #[serde(default)]
    pub undefined_should_contribute: bool,
    #[serde(default)]
    pub manual_reference: Option<GroundPosition>,
}

impl QcConfig {
    pub fn set_report_type(&mut self, report_type: QcReportType) {
        self.report = report_type;
    }
    pub fn set_prefered_radio_orbit(&mut self) {
        self.prefered_orbit = PreferedOrbit::BrdcRadio;
    }
    pub fn set_prefered_sp3_orbit(&mut self) {
        self.prefered_orbit = PreferedOrbit::SP3;
    }
    pub fn set_reference_position(&mut self, pos: GroundPosition) {
        self.manual_reference = Some(pos.clone());
    }
    pub fn undefined_should_contribute(&mut self) {
        self.undefined_should_contribute = true;
    }
}

impl Render for QcConfig {
    fn render(&self) -> Markup {
        html! {
            tr {
                td {
                    "Report"
                }
                td {
                    (self.report.to_string())
                }
            }
            @if let Some(position) = self.manual_reference {
                tr {
                    td {
                        (position.render())
                    }
                }
            }
        }
    }
}
