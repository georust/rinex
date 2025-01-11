use crate::report::obs::QcObservationsReport;

use crate::{
    context::{meta::MetaData, QcContext},
    prelude::{html, Markup, Render},
};

use std::collections::HashMap;

/// RINEX Observation Report shared by both ROVERs and BASEs
pub struct QcBasesObservationsReport {
    pub reports: HashMap<MetaData, QcObservationsReport>,
}

impl QcBasesObservationsReport {
    pub fn new(ctx: &QcContext) -> Self {
        let mut reports = HashMap::new();
        for (k, v) in ctx.obs_dataset.iter() {
            if !k.is_rover {
                reports.insert(k.meta.clone(), QcObservationsReport::new(&v));
            }
        }
        Self { reports }
    }
}

impl Render for QcBasesObservationsReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" id="qc-base-observations" style="display:none" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Base Stations"
                            }
                            td {
                                select id="qc-base-obs-selector" onclick="onQcBaseObsSelection()" {
                                    @ for base in self.reports.keys() {
                                        option value=(base.name) {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
