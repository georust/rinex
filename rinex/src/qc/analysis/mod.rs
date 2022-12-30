use crate::prelude::*;
use super::HtmlReport;
use horrorshow::{helper::doctype, RenderBox};

mod sampling;
use sampling::QcSamplingAnalysis;

mod obs;
use obs::QcObsAnalysis;

mod sv;
use sv::QcSvAnalysis;

#[derive(Debug, Clone)]
pub struct QcAnalysis {
	pub classifier: Constellation,
	sv: QcSvAnalysis,
	observ: QcObsAnalysis,
	sampling: QcSamplingAnalysis,
}

impl QcAnalysis {
	pub fn new(classifier: Constellation, rnx: &Rinex) -> Self {
		Self {
			classifier,
			sv: QcSvAnalysis::new(rnx),
			observ: QcObsAnalysis::new(rnx),
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
                        meta(charset="utf-8");
						meta(name="viewport", content="width=device-width, initial-scale=1");
						link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        title {
							: format!("{} QC Analysis", self.classifier)
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
