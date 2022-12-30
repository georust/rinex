use crate::prelude::*;
use super::HtmlReport;
use horrorshow::{helper::doctype, RenderBox};

mod sampling;
use sampling::QcSamplingAnalysis;

#[derive(Debug, Clone)]
pub struct QcAnalysis {
	pub classifier: Constellation,
	sampling: QcSamplingAnalysis,
}

impl QcAnalysis {
	pub fn new(classifier: Constellation, rnx: &Rinex) -> Self {
		Self {
			classifier,
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
				h1(class="title") {
					: self.classifier.to_string()
				}
				div(id="sampling report") {
					h1(class="title") {
						: "Sampling"
					}
					table(class="table is-bordered") {
						: self.sampling.to_inline_html()
					}
				}
			}
		}
	}
}
