use crate::context::{meta::ObsMetaData, QcContext};
use rinex::prelude::Rinex;

pub struct QcBaseSummary {}

impl QcBaseSummary {
    pub fn new(ctx: &QcContext, meta: &ObsMetaData, rinex: &Rinex) -> Self {
        Self {}
    }
}
