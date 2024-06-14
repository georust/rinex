use thiserror::Error;
use qc_traits::html::*;
use rinex::prelude::*;
use rinex::{geodetic, wgs84};

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
    InvalidReportType,
}

use std::fmt::Display;
use std::str::FromStr;

/// [ReportType]
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QcReportType {
    /// In [Summary] mode, only the summary section
    /// of the report is to be generated. It is the lightest
    /// form we can generate.
    Summary,
    /// In [Light] mode, we have one analysis per input [ProductType].
    #[Default]
    Light,
    /// In [Full] mode, we generate the [CombinedReport] as well,
    /// which results from the consideration of all input [ProductType]s
    /// at the same time.
    Full,
}

impl FromStr for ReportType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().lowercase() {
            "sum" | "summ" | "summary" => Ok(Self::Summary),
            "light" => Ok(Self::Light),
            "full" => Ok(Self::Full),
            _ => Err(Error::InvalidReportType),
        }
    }
}

impl Display for ReportType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QcClassification::GNSS => f.write_str("GNSS Constellations"),
            QcClassification::SV => f.write_str("Satellite Vehicles"),
            QcClassification::Physics => f.write_str("Physics"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum CsStrategy {
    /// Study CS events and report them
    #[default]
    Study,
    /// Study CS events and repair them
    StudyAndRepair,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct ProcessingOpts {
    /// Cs analysis/reporting strategy
    pub cs: CsStrategy,
    /// Ionospheric variation tolerance
    pub iono_rate_tolerance: f64,
    pub iono_rate_tolerance_dt: Duration,
    /// Clock Drift Moving average window slot
    pub clock_drift_window: Duration,
    /// Increment of the elelavtion mask
    pub elev_mask_increment: f64,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct QcConfig {
    #[cfg_attr(feature = "serde", serde(default))]
    pub report: QcReportType,
    #[cfg_attr(feature = "serde", serde(default))]
    pub manual_ecef_reference: Option<(f64, f64, f64)>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub manual_geo_reference: Option<(f64, f64, f64)>,
}

impl QcOpts {
    pub fn set_report_type(&mut self, report_type: QcReportType) {
        self.report = report_type;
    }
    pub fn set_geo_reference(&self, geo: (f64, f64, f64)) -> Self {
        self.manual_geo_reference = Some(geo);
    }
    pub fn set_ecef_reference(&self, ecef: (f64, f64, f64)) -> Self {
        self.manual_ecef_reference = Some(ecef);
    }
}

impl RenderHtml for QcOpts {
    fn to_html(&self) -> String {
        panic!("qcopts cannot be rendered on its own")
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "Classification"
                }
                th {
                    : format!("{}", self.classification)
                }
            }
            tr {
                th {
                    : "Min. SNR"
                }
                td {
                    : format!("{} dB", self.min_snr_db)
                }
            }
            tr {
                th {
                    : "Elevation mask"
                }
                @ if let Some(mask) = self.elev_mask {
                    td {
                        : format!("{} °", mask)
                    }
                } else {
                    td {
                        : "None"
                    }
                }
            }
            tr {
                th {
                    : "Data gap"
                }
                @ if let Some(tol) = self.gap_tolerance {
                    td {
                        : format!("{} tolerance", tol)
                    }
                } else {
                    td {
                        : "No tolerance"
                    }
                }
            }
            tr {
                th {
                    : "Clock Drift Window"
                }
                td {
                    : self.clock_drift_window.to_string()
                }
            }
        }
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn qc_opts_serdes() {
        let content = r#"
			{
				"classification": "GNSS"
			}"#;
        let _opts: QcOpts = serde_json::from_str(content).unwrap();

        let content = r#"
			{
				"classification": "SV"
			}"#;
        let _opts: QcOpts = serde_json::from_str(content).unwrap();

        /*let content = r#"
            {
                "statistics": {
                    "window": "10 seconds"
                }
            }"#;

        let opts: QcOpts = serde_json::from_str(content).unwrap();
        assert_eq!(opts.reporting, ReportingStrategy::PerSv);
        assert_eq!(opts.statistics, Some(StatisticsOpts {
            window: Slot::Duration(Duration::from_seconds(10.0)),
        }));
        assert!(opts.processing.is_none());

        let content = r#"
            {
                "statistics": {
                    "window": "10 %"
                }
            }"#;

        let opts: QcOpts = serde_json::from_str(content).unwrap();
        assert_eq!(opts.reporting, ReportingStrategy::PerSignal);
        assert_eq!(opts.statistics, Some(StatisticsOpts {
            window: Slot::Percentage(10.0_f64),
        }));
        assert!(opts.processing.is_none());
        */
    }
}
