use crate::cli::Context;
use std::collections::HashMap;

use gnss_rtk::prelude::{
    AprioriPosition, Arc, Bodies, Cosm, Duration, Epoch, Frame,
    InterpolationResult as RTKInterpolationResult, LightTimeCalc, TimeScale, Vector3, SV,
};

use rinex::navigation::Ephemeris;

pub struct Orbit<'a> {
    latest: HashMap<SV, Ephemeris>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, Ephemeris)> + 'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let brdc = ctx
            .data
            .brdc_navigation()
            .expect("BRDC navigation required");
        Self {
            latest: HashMap::with_capacity(64),
            iter: Box::new(brdc.ephemeris.peekable()),
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<RTKInterpolationResult> {
        None
    }
}
