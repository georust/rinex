use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use crate::cfg::ConfigError;

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

impl std::str::FromStr for QcReportType {
    type Err = ConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "sum" | "summ" | "summary" => Ok(Self::Summary),
            "full" => Ok(Self::Full),
            _ => Err(ConfigError::ReportType),
        }
    }
}

impl std::fmt::Display for QcReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Full => f.write_str("Full"),
            Self::Summary => f.write_str("Summary"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcReportOpts {
    #[serde(default, rename(deserialize = "type"))]
    pub report_type: QcReportType,
}

impl Render for QcReportOpts {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            "Report Type"
                        }
                        td {
                            (self.report_type.to_string())
                        }
                    }
                }
            }
        }
    }
}
