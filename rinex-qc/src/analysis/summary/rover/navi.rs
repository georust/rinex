use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::Rinex,
};

use rinex::prelude::Constellation;

use std::collections::HashMap;

pub struct QcNavConstellationSummary {
    pub brdc_strategy_compatible: bool,
    pub ppp_strategy_compatible: bool,
    pub ultra_ppp_strategy_compatible: bool,
}

pub struct QcNaviSummary {
    pub html_id: String,
    pub tropo_model_optimization: bool,
    pub constellations_navi: HashMap<Constellation, QcNavConstellationSummary>,
}

impl QcNaviSummary {
    pub fn new(ctx: &QcContext, obs_meta: &ObsMetaData, rover: &Rinex) -> Self {
        Self {
            html_id: obs_meta.meta.name.to_string(),
            tropo_model_optimization: ctx.allows_troposphere_model_optimization(&obs_meta.meta),
            constellations_navi: {
                let mut constellations_sum = HashMap::new();
                for constellation in rover.constellations_iter() {
                    let nav_constellations = if let Some(rinex) = &ctx.nav_dataset {
                        rinex.constellations_iter().collect::<Vec<_>>()
                    } else {
                        vec![]
                    };

                    let brdc_strategy_compatible = nav_constellations.contains(&constellation);
                    let mut ppp_strategy_compatible = false;
                    let mut ultra_ppp_strategy_compatible = false;

                    #[cfg(feature = "sp3")]
                    if brdc_strategy_compatible {
                        // TODO SP3 support
                    }

                    let sum = QcNavConstellationSummary {
                        brdc_strategy_compatible,
                        ppp_strategy_compatible,
                        ultra_ppp_strategy_compatible,
                    };

                    constellations_sum.insert(constellation, sum);
                }
                constellations_sum
            },
        }
    }
}
