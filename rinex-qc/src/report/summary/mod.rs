mod base;
mod general;
mod rover;

use base::QcBaseSummary;
use general::QcGeneralSummary;
use rover::QcRoverSummary;

use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::{html, Markup, Render},
};

use std::collections::HashMap;

pub struct QcSummary {
    general: QcGeneralSummary,
    rovers_sum: HashMap<ObsMetaData, QcRoverSummary>,
    bases_sum: HashMap<ObsMetaData, QcBaseSummary>,
}

impl QcSummary {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            general: QcGeneralSummary::new(ctx),
            rovers_sum: {
                let mut rovers = HashMap::new();
                for (meta, rover) in ctx.obs_dataset.iter() {
                    if meta.is_rover {
                        let meta = meta.clone();
                        let rover_summary = QcRoverSummary::new(ctx, &meta, &rover);
                        rovers.insert(meta.clone(), rover_summary);
                    }
                }
                rovers
            },
            bases_sum: {
                let mut bases = HashMap::new();
                for (meta, base) in ctx.obs_dataset.iter() {
                    if !meta.is_rover {
                        let meta = meta.clone();
                        let base_summary = QcBaseSummary::new(ctx, &meta, &base);
                        bases.insert(meta.clone(), base_summary);
                    }
                }
                bases
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
                        tr {
                            @ if self.bases_sum.len() == 0 {
                                @ if self.rovers_sum.len() == 1 {
                                    th class="is-info" {
                                        "Rover"
                                    }
                                } @ else {
                                    th class="is-info" {
                                        "Rovers"
                                    }
                                }
                            }
                        }
                        @ for (_, rover) in self.rovers_sum.iter() {
                            tr {
                                td {
                                    (rover.render())
                                }
                            }
                        }

                        @ if !self.bases_sum.is_empty() {
                            tr {
                                td {
                                    @ if self.bases_sum.len() == 1 {
                                        th class="is-info" {
                                            "Base Station"
                                        }
                                    } @ else {
                                        th class="is-info" {
                                            "Base Stations"
                                        }
                                    }
                                }
                            }
                            @ for (_, base) in self.bases_sum.iter() {
                                tr {
                                    td {
                                        (base.render())
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
