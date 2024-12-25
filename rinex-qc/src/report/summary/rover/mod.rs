mod nav_post;
use nav_post::QcNavPostSummary;

mod bias;
use bias::QcBiasSummary;

mod timeframe;
use timeframe::QcTimeFrame;

use rinex::prelude::Constellation;
use std::collections::HashMap;

use crate::{
    context::QcContext,
    prelude::{html, Render, Markup},
};

/// [QcRoverSummary] is a general report, per rover in the dataset
pub struct QcRoverSummary {
    /// NAVI summary
    navi: QcNavPostSummary,
    /// BIAS summary
    bias_sum: QcBiasSummary,
    /// QcTimeFrames per Constellation
    timeframes: HashMap<Constellation, QcTimeFrame>,
}

impl QcRoverSummary {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            navi: QcNavPostSummary::new(ctx),
            bias_sum: QcBiasSummary::new(ctx),
            timeframes: {
                let mut timeframes = HashMap::new();
                for (meta, rinex) in ctx.obs_dataset.iter() {
                    for constellation in rinex.constellations_iter() {
                        timeframes.insert(
                            constellation,
                            QcTimeFrame::new(constellation, ctx, meta, rinex),
                        );
                    }
                }
                timeframes
            },
        }
    }
}
impl Render for QcRoverSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
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
                        tr {
                            th class="is-info" {
                                "Time Frame"
                            }
                            td {
                                (self.time_frame.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
