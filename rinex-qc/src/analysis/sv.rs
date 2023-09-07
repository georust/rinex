use super::{pretty_array, QcOpts};
<<<<<<< HEAD:rinex-qc/src/analysis/sv.rs
use rinex::prelude::Rinex;

use rinex_qc_traits::HtmlReport;
use horrorshow::{box_html, RenderBox};
=======
use crate::prelude::*; //table_lengthy_td
>>>>>>> main:rinex/src/qc/analysis/sv.rs

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
        panic!("sv analysis cannot be rendered on its own")
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
