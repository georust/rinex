use super::prelude::*;
use horrorshow::helper::doctype;
use strum_macros::EnumString;

mod sampling;
//mod advanced;
//mod navigation;
//mod observation;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QcType {
    /// Basic QC only performs
    /// Sampling and observation integrity analysis
    Basic,
    /// Intermediate QC integrates
    /// the Basic QC, and performs
    /// basic studies on provided Observations,
    /// like Code biases estimation.
    /// If Navigation Context is provided,
    /// it is used for basic yet enhanced Observation analysis.
    Intermediate,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Grade {
    #[strum(serialize = "A++")]
    GradeApp,
    #[strum(serialize = "A+")]
    GradeAp,
    #[strum(serialize = "A")]
    GradeA,
    #[strum(serialize = "B")]
    GradeB,
    #[strum(serialize = "C")]
    GradeC,
    #[strum(serialize = "D")]
    GradeD,
    #[strum(serialize = "E")]
    GradeE,
    #[strum(serialize = "F")]
    GradeF,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    /// Sampling QC
    pub sampling: sampling::QcReport,
    /*
        /// Observation RINEX specific QC
        pub observation: Option<ObservationQc>,
        /// Navigation RINEX specific QC
        pub navigation: Option<NavigationQc>,
        /// Advanced Observation + Navigation specific QC
        pub advanced: Option<AdvancedQc>,
    */
}

/*
pub struct QcOpts{}

impl Default for QcOpts {
    fn default() -> Self {
        QcOpts {}
    }
}*/

impl QcReport {
    /// Processes given RINEX and generates a summary report.
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>, qc_type: QcType) -> Self {
        match qc_type {
            QcType::Basic => Self::basic_qc(rnx, nav),
            QcType::Intermediate => Self::intermediate_qc(rnx, nav),
        }
    }
    fn basic_qc(rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        Self {
            sampling: sampling::QcReport::new(rnx),
            /*observation: None,
            navigation: None,
            advanced: None,*/
        }
    }
    fn intermediate_qc(rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        Self::basic_qc(rnx, nav)
    }
    /// Dumps self into (self sufficient) HTML
    pub fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        title: "RINEX QC summary";
                        style {
                            table {
                                font-family: "arial, sans-serif";
                                border-collapse: "collapse";
                                width: "100%";
                            }
                            td {
                                border: "1px solid #dddddd";
                                text-align: "left";
                                padding: "8px";
                            }
                            th {
                                border: "1px solid #dddddd";
                                text-align: "left";
                                padding: "8px";
                            }
                            /*tr:nth-child(event) {
                                background-color: "#dddddd";
                            }*/
                        }
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    /// Dumps self into HTML <div> section, named as suggested
    pub fn to_inline_html(&self) -> String {
        format!(
            "{}",
            html! {
                h2(id="heading") {
                    : "RINEX Quality Check summary"
                }
                h4(id="version") {
                    program-version: format!("rust-rnx: v{}", env!("CARGO_PKG_VERSION"))
                }
                div(id="sampling") {
                    : self.sampling.to_html()
                }
            }
        )
    }
}
