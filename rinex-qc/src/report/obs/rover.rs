use crate::report::obs::QcObservationsReport;

use crate::{
    context::{meta::MetaData, QcContext},
    prelude::{html, Markup, Render},
};

use std::collections::HashMap;

/// RINEX Observation Report shared by both ROVERs and BASEs
pub struct QcRoversObservationsReport {
    pub reports: HashMap<MetaData, QcObservationsReport>,
}

impl QcRoversObservationsReport {
    pub fn new(ctx: &QcContext) -> Self {
        let mut reports = HashMap::new();
        for (k, v) in ctx.obs_dataset.iter() {
            if k.is_rover {
                reports.insert(k.meta.clone(), QcObservationsReport::new(&v));
            }
        }
        Self { reports }
    }
}

impl Render for QcRoversObservationsReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "File Set"
                            }
                            td {
                            }
                        }
                    }
                }
            }
        }
    }
}
