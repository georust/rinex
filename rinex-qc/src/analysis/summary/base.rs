use crate::context::{meta::ObsMetaData, QcContext};

use rinex::prelude::Rinex;

pub struct QcBaseSummary {}

impl QcBaseSummary {
    pub fn new(_ctx: &QcContext, _meta: &ObsMetaData, _rinex: &Rinex) -> Self {
        Self {}
    }
}
