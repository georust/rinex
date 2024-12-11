use maud::{html, Markup, Render};
use rinex::prelude::{GroundPosition, TimeScale};

use crate::prelude::{QcConfig, QcContext};

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
    pub fn new(context: &QcContext, cfg: &QcConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            navi: QcNavPostSummary::new(context),
            bias_sum: QcBiasSummary::new(context),
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
                                button aria-label="Context / Dataset compliancy" data-balloon-pos="right" {
                                    "Compliancy"
                                }
                            }
                            td {
                                (self.navi.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Physical and Environmental bias analysis & cancellation capabilities" data-balloon-pos="right" {
                                    "Bias"
                                }
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
