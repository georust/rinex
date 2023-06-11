use super::{pretty_array, QcOpts};
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct QcSvAnalysis {
    pub sv: Vec<String>,
}

impl QcSvAnalysis {
    pub fn new(rnx: &Rinex, _nav: &Option<Rinex>, _opts: &QcOpts) -> Self {
        let mut sv = rnx.space_vehicles();
        sv.sort();
        Self {
            sv: { sv.iter().map(|sv| sv.to_string()).collect() },
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::RenderBox;

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
                    : pretty_array(&self.sv)
                }
            }
        }
    }
}
