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

impl Default for ProcessingOpts {
    fn default() -> Self {
        Self {
            cs: CsStrategy::default(),
            iono_rate_tolerance: 400.0E-2_f64,
            iono_rate_tolerance_dt: Duration::from_seconds(60.0_f64),
            clock_drift_window: Duration::from_seconds(600.0_f64),
            elev_mask_increment: 10.0_f64,
        }
    }
}

/// Qc Report classification method
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Classify the QC report by desired data set
pub enum QcClassification {
    /// Report per GNSS system
    #[default]
    GNSS,
    /// Report per SV
    SV,
    /// Report per Physics (Observable, Orbit..)
    Physics,
}

impl std::fmt::Display for QcClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QcClassification::GNSS => f.write_str("GNSS Constellations"),
            QcClassification::SV => f.write_str("Satellite Vehicles"),
            QcClassification::Physics => f.write_str("Physics"),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct QcOpts {
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(docs, doc(cfg(feature = "processing")))]
    pub classification: QcClassification,
    /// Minimum SNR level to consider in our analysis.
    /// For example, this is used when determining whether
    /// an epoch is "complete" or not.
    #[cfg_attr(feature = "serde", serde(default))]
    pub min_snr_db: f64,
    /// Elevation mask
    pub elev_mask: Option<f64>,
    /// Min. duration tolerated, so it is not reported as a data gap.
    /// If None: dominant sample rate is prefered.
    pub gap_tolerance: Option<Duration>,
    /// Manually defined Ground position (ECEF)
    pub ground_position: Option<GroundPosition>,
    /// Window duration to be used, during RX clock drift analysis
    #[cfg_attr(feature = "serde", serde(default = "default_drift_window"))]
    pub clock_drift_window: Duration,
}

impl QcOpts {
    pub fn with_classification(&self, classification: QcClassification) -> Self {
        let mut s = self.clone();
        s.classification = classification;
        s
    }

    pub fn with_min_snr(&self, snr_db: f64) -> Self {
        let mut s = self.clone();
        s.min_snr_db = snr_db;
        s
    }

    pub fn with_ground_position_ecef(&self, pos: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.ground_position = Some(wgs84!(pos.0, pos.1, pos.2));
        s
    }

    pub fn with_ground_position_geo(&self, pos: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.ground_position = Some(geodetic!(pos.0, pos.1, pos.2));
        s
    }
}

fn default_drift_window() -> Duration {
    Duration::from_seconds(3600.0)
}

impl Default for QcOpts {
    fn default() -> Self {
        Self {
            gap_tolerance: None,
            ground_position: None,
            min_snr_db: 20.0, // dB
            elev_mask: None,
            classification: QcClassification::default(),
            clock_drift_window: default_drift_window(),
        }
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
