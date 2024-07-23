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
    name: String,
    /// Configuration used
    cfg: QcConfig,
    /// NAVI summary
    pub navi: QcNavPostSummary,
    /// Main timescale
    timescale: Option<TimeScale>,
    /// BIAS summary
    bias_sum: QcBiasSummary,
    /// reference position
    reference_position: Option<GroundPosition>,
}

impl QcSummary {
    pub fn new(context: &QcContext, cfg: &QcConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            name: context.name(),
            timescale: context.timescale(),
            bias_sum: QcBiasSummary::new(context),
            navi: QcNavPostSummary::new(context),
            reference_position: context.reference_position(),
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
                            th class="is-info is-bordered" {
                                (self.name.clone())
                            }
                        }
                        tr {
                            th {
                                "Timescale"
                            }
                            @if let Some(timescale) = self.timescale {
                                td {
                                    (timescale.to_string())
                                }
                            } @else {
                                td {
                                    "Not Applicable"
                                }
                            }
                        }
                        tr {
                            @if let Some(position) = self.cfg.manual_reference {
                                th {
                                    "(Manual) Reference position"
                                }
                                td {
                                    (position.render())
                                }
                            } @else if let Some(position) = self.reference_position {
                                th {
                                    "Reference position"
                                }
                                td {
                                    (position.render())
                                }
                            } @else {
                                th {
                                    "Reference position"
                                }
                                td {
                                    "None"
                                }
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
