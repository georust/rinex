use super::QcOpts;
use qc_traits::html::*;
use rinex::prelude::{Rinex, SV};

#[derive(Debug, Clone)]
pub struct QcSvAnalysis {
    pub sv: Vec<SV>,
}

use itertools::Itertools;

impl QcSvAnalysis {
    pub fn new(primary: &Rinex, _opts: &QcOpts) -> Self {
        let sv: Vec<_> = primary.sv().collect();
        Self { sv }
    }
}

impl RenderHtml for QcSvAnalysis {
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
                    p {
                        @ for mut chunks in &self.sv.iter().chunks(12) {
                            p {
                                @ while let Some(sv) = chunks.next() {
                                    : format!("{:x}, ", sv)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
