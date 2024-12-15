use maud::{html, Markup, Render};
use rinex::prelude::{GroundPosition, TimeScale};

use crate::{cfg::QcConfig, context::QcContext};

mod nav_post;
use nav_post::QcNavPostSummary;

mod bias;
use bias::QcBiasSummary;

/// [QcSummary] is the lightest report form,
/// sort of a report introduction that will always be generated.
/// It only gives high level and quick description.
pub struct QcSummary {
    /// Configuration used
    cfg: QcConfig,
    /// NAVI summary
    pub navi: QcNavPostSummary,
    /// BIAS summary
    bias_sum: QcBiasSummary,
}

impl QcSummary {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            cfg: ctx.cfg.clone(),
            navi: QcNavPostSummary::new(ctx),
            bias_sum: QcBiasSummary::new(ctx),
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
                            th class="is-info" {
                                "QC Settings"
                            }
                            td {
                                (self.cfg.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Compliancy"
                            }
                            td {
                                (self.navi.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Bias"
                            }
                            td {
                                (self.bias_sum.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
