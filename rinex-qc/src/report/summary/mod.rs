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
            timescale: context.timescale(),
            bias_sum: QcBiasSummary::new(context),
            navi: QcNavPostSummary::new(context),
            reference_position: context.reference_position(),
            name: context.primary_name().unwrap_or("Undefined".to_string()),
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
                                button aria-label="Timescale in which samples observation are expressed.
        Navigation solutions are expressed in this timescale by default." data-balloon-pos="right" {
                                    "Timescale"
                                }
                            }
                            @if let Some(timescale) = self.timescale {
                                td {
                                    (timescale.to_string())
                                }
                            } @else {
                                td {
                                    button aria-label="This dataset is not a timeserie." data-balloon-pos="up" {
                                        "Not Applicable"
                                    }
                                }
                            }
                        }
                        tr {
                            @if let Some(position) = self.cfg.manual_reference {
                                th {
                                    button aria-label="Ground based reference position" data-balloon-pos="up" {
                                        "(Manual) Reference position"
                                    }
                                }
                                td {
                                    button aria-label="Provided by custom command line" data-balloon-pos="up" {
                                        (position.render())
                                    }
                                }
                            } @else if let Some(position) = self.reference_position {
                                th {
                                    button aria-label="Ground based (RX) position" data-balloon-pos="up" {
                                        "Reference position"
                                    }
                                }
                                td {
                                    button aria-label="Parsed from RINEX header" data-balloon-pos="up" {
                                        (position.render())
                                    }
                                }
                            } @else {
                                th {
                                    button aria-label="Ground based reference position" data-balloon-pos="up" {
                                        "Reference position"
                                    }
                                }
                                td {
                                    button aria-label="Compass projection is disabled.
Most navigation geometric/attibutes filter cannot apply.
Initial survey/guess is implied." data-balloon-pos="up" {
                                        "None"
                                    }
                                }
                            }
                        }
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
