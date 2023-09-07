use crate::prelude::*;
use super::{pretty_array, QcOpts};

use horrorshow::RenderBox;
use rinex_qc_traits::HtmlReport;

#[derive(Debug, Clone)]
pub struct QcSvAnalysis {
    pub sv: Vec<String>,
}

impl QcSvAnalysis {
    pub fn new(primary: &Rinex, _opts: &QcOpts) -> Self {
        let sv = primary.sv();
        Self {
            sv: { sv.map(|sv| sv.to_string()).collect() },
        }
    }
}

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
