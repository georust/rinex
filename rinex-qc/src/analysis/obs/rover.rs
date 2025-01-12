use crate::analysis::obs::QcObservationsAnalysis;

use crate::context::{meta::MetaData, QcContext};

use std::collections::HashMap;

/// RINEX Observation Report shared by both ROVERs and BASEs
pub struct QcRoversObservationAnalysis {
    pub analysis: HashMap<MetaData, QcObservationsAnalysis>,
}

impl QcRoversObservationAnalysis {
    pub fn is_null(&self) -> bool {
        for (_, analysis) in self.analysis.iter() {
            if !analysis.is_null() {
                return false;
            }
        }
        true
    }

    pub fn new(ctx: &QcContext) -> Self {
        let mut analysis = HashMap::new();
        for (obs_meta, v) in ctx.obs_dataset.iter() {
            if obs_meta.is_rover {
                analysis.insert(
                    obs_meta.meta.clone(),
                    QcObservationsAnalysis::new(&obs_meta.meta.name, &v),
                );
            }
        }
        Self { analysis }
    }
}
