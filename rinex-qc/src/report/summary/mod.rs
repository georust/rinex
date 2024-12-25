mod general;
mod rover;

use general::QcGeneralSummary;
use rover::QcRoverSummary;

use crate::{
    context::{meta::MetaData, QcContext},
    prelude::{html, Markup, Render},
};

use std::collections::HashMap;

pub struct QcSummary {
    general: QcGeneralSummary,
    rovers_sum: HashMap<MetaData, QcRoverSummary>,
}

impl QcSummary {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            general: QcGeneralSummary::new(ctx),
            rovers_sum: {
                let mut rovers = HashMap::new();
                for (meta, rinex) in ctx.obs_dataset.iter() {
                    let meta = meta.clone();
                    let rover_summary = QcRoverSummary::new(ctx, &meta, &rinex);
                    rovers.insert(meta.clone(), rover_summary);
                }
                rovers
            },
        }
    }
}

impl Render for QcSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            td {
                                (self.general.render())
                            }
                        }
                        @ for (meta, rover) in self.rovers_sum.iter() {
                            tr {
                                td {
                                    (rover.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
