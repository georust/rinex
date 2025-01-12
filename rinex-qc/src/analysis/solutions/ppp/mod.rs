use crate::prelude::{html, Markup, Render};

mod summary;
use summary::Summary;

use crate::prelude::QcContext;
use gnss_rtk::prelude::{Epoch, PVTSolution};

#[derive(Default)]
pub struct QcNavPostPPPSolutions {
    summary: Summary,
}

impl QcNavPostPPPSolutions {
    pub fn new_solution(&mut self, t: Epoch, solution: PVTSolution) {
        self.summary.new_solution(t, solution)
    }
}

impl Render for QcNavPostPPPSolutions {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Summary"
                            }
                            td {
                                (self.summary.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
