mod navi;
use navi::QcNaviSummary;

// mod timeframe;
// use timeframe::QcTimeFrame;

use rinex::prelude::Constellation;
use std::collections::HashMap;

use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::{html, Markup, Render, Rinex},
};

/// [QcRoverSummary] is a general report, per rover in the dataset
pub struct QcRoverSummary {
    /// NAVi summary
    navi: QcNaviSummary,
    //     /// QcTimeFrames per Constellation
    //     timeframes: HashMap<Constellation, QcTimeFrame>,
}

impl QcRoverSummary {
    pub fn new(ctx: &QcContext, meta: &ObsMetaData, rover_rinex: &Rinex) -> Self {
        Self {
            navi: QcNaviSummary::new(ctx, meta, &rover_rinex),
            // timeframes: {
            //     let mut timeframes = HashMap::new();
            //     // for (meta, rinex) in ctx.obs_dataset.iter() {
            //     //     for constellation in rinex.constellations_iter() {
            //     //         timeframes.insert(
            //     //             constellation,
            //     //             QcTimeFrame::new(constellation, ctx, meta, rinex),
            //     //         );
            //     //     }
            //     // }
            //     timeframes
            // },
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
                                button aria-label="NB: summary report does not account for the time frame,
only files & constellations relationship.\n
Use the timeframe analysis to actually confirm the summary report"
                                data-balloon-pos="right" {
                                    b {
                                        "Compliancy"
                                    }
                                }
                            }
                            td {
                                (self.navi.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
