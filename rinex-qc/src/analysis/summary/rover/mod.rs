pub(crate) mod navi;
use navi::QcNaviSummary;

// mod timeframe;
// use timeframe::QcTimeFrame;

use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::Rinex,
};

/// [QcRoverSummary] is a general report, per rover in the dataset
pub struct QcRoverSummary {
    /// NAVi summary
    pub navi: QcNaviSummary,
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
