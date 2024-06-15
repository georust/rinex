use qc_traits::html::*;
use rinex::prelude::*;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{
    //de::Error,
    //Serializer,
    Deserialize,
    //Deserializer,
    Serialize,
};

/// Configuration Error
#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("invalid report type")]
    InvalidReportType,
}

use std::fmt::Display;
use std::str::FromStr;

/// [QcReportType]
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QcReportType {
    /// In [Summary] mode, only the summary section
    /// of the report is to be generated. It is the lightest
    /// form we can generate.
    Summary,
    /// In [Full] mode, we generate the [CombinedReport] as well,
    /// which results from the consideration of all input [ProductType]s
    /// at the same time.
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

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct QcConfig {
    #[cfg_attr(feature = "serde", serde(default))]
    pub report: QcReportType,
    #[cfg_attr(feature = "serde", serde(default))]
    pub manual_reference: Option<GroundPosition>,
}

impl QcConfig {
    pub fn set_report_type(&mut self, report_type: QcReportType) {
        self.report = report_type;
    }
    pub fn set_reference_position(&mut self, pos: GroundPosition) {
        self.manual_reference = Some(pos.clone());
    }
}

impl RenderHtml for QcConfig {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                td {
                    : "Report"
                }
                td {
                    : self.report.to_string()
                }
            }
            @ if let Some(position) = self.manual_reference {
                tr {
                    td {
                        : position.to_inline_html()
                    }
                }
            }
        }
    }
}
