use crate::prelude::*;
use crate::processing::TargetItem;

use super::{pretty_array, HtmlReport, QcOpts};
use horrorshow::{helper::doctype, RenderBox};

mod sv;

mod obs;
use obs::QcObsAnalysis;

//mod antenna;
mod sampling;

use sampling::QcSamplingAnalysis;
use sv::QcSvAnalysis;

#[derive(Debug, Clone)]
pub struct QcAnalysis {
    pub classifier: TargetItem,
    sv: QcSvAnalysis,
    observ: QcObsAnalysis,
    sampling: QcSamplingAnalysis,
}

impl QcAnalysis {
    pub fn new(classifier: TargetItem, rnx: &Rinex, nav: &Option<Rinex>, opts: &QcOpts) -> Self {
        Self {
            classifier,
            sv: QcSvAnalysis::new(rnx, nav, opts),
            #[cfg(feature = "obs")]
            observ: QcObsAnalysis::new(rnx, nav, opts),
            sampling: QcSamplingAnalysis::new(rnx),
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
                            : format!("{:?} QC Analysis", self.classifier)
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
