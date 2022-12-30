use crate::prelude::*;
use crate::Carrier;
use crate::Observable;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QcSvAnalysis {
	pub sv: Vec<String>
}

impl QcSvAnalysis {
    pub fn new(rnx: &Rinex) -> Self {
		let sv = rnx
			.space_vehicules();
        Self {
			sv: {
				sv.iter()
					.map(|sv| sv.to_string())
					.collect()
			}
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::{helper::doctype, RenderBox};

impl HtmlReport for QcSvAnalysis {
	fn to_html(&self) -> String {
		todo!()
	}
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
			tr {
				th {
					: "PRN#"
				}
				td {
					: format!("{:?}", self.sv)
				}
			}
        }
    }
}
