use super::{pretty_array, HtmlReport, QcOpts};
use crate::prelude::*;

#[cfg(feature = "processing")]
use crate::preprocessing::TargetItem;

use horrorshow::{helper::doctype, RenderBox};

mod sv;

mod obs;
use obs::QcObsAnalysis;

mod sampling;

use sampling::QcSamplingAnalysis;
use sv::QcSvAnalysis;

#[derive(Debug, Clone)]
/// RINEX File Quality analysis report
pub struct QcAnalysis {
    /// RINEX file sampling analysis
    ///   - dominant sample rate
    ///   - data gaps, etc..
    sampling: QcSamplingAnalysis,
    /// [crate::Sv] specific analysis
    ///  - identifies, PRN# versus time
    ///  - Rise and Fall datetime, etc..
    sv: QcSvAnalysis,
    /// [crate::observation::Record] specific analysis,
    /// is truly complete when both "obs" and "processing"
    /// features are enabled
    observ: QcObsAnalysis,
}

impl QcAnalysis {
    /// Creates a new Analysis Report from given RINEX context.  
    ///   - `rnx` : primary file
    ///   - `nav` : Optional secondary file to augment feasible anlysis
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>, opts: &QcOpts) -> Self {
        Self {
            sv: QcSvAnalysis::new(rnx, nav, opts),
            sampling: QcSamplingAnalysis::new(rnx),
            #[cfg(feature = "obs")]
            observ: QcObsAnalysis::new(rnx, nav, opts),
        }
    }
}

impl HtmlReport for QcAnalysis {
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(content="text/html", charset="utf-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        title {
                            : "RINEX QC analysis"
                        }
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(id="analysis") {
                div(id="sampling") {
                    table(class="table is-bordered") {
                        thead {
                            th {
                                : "Sampling"
                            }
                        }
                        tbody {
                            : self.sampling.to_inline_html()
                        }
                    }
                }
                div(id="sv") {
                    table(class="table is-bordered") {
                        thead {
                            th {
                                : "Sv"
                            }
                        }
                        tbody {
                            : self.sv.to_inline_html()
                        }
                    }
                }
                div(id="observations") {
                    table(class="table is-bordered") {
                        thead {
                            th {
                                : "Observations"
                            }
                        }
                        tbody {
                            : self.observ.to_inline_html()
                        }
                    }
                }
            }
        }
    }
}
