use crate::context::QcContext;

use rinex::prelude::{
    SV, 
    Observable,
    Epoch,
};

pub struct SVPosition {
    pub sv: SV,
    pub x_km_ecef: f64,
    pub y_km_ecef: f64,
    pub z_km_ecef: f64,
}

pub struct SVSignal {
    pub signal: Observable,
    pub value: f64,
}

/// Iterated item
pub enum Item {
    /// [SVPosition]
    SVPosition(SVPosition),
    /// [SVSignal]
    SVSignal(SVSignal),
}

/// [PPPIter] is our main [Iterator] that can be built from [QcContext]
/// and is specifically dedicated to generate post processed [Solutions].
pub struct PPPIter {
    solver: Solver,
    cggtts: bool,
}

impl PPPIter {
    pub fn new_pvt(ctx: QcContext, cfg: Config) -> Self {
        Self {
            solver: Solver::new_almanac_frame(
                &cfg,
                ctx.apriori(),
                ctx.almanac,
                ctx.earth_cef,
                orbits: ctx.orbits_iter(),
            ),
            cggtts: false,
        }
    }
    pub fn new_cggtts(ctx: QcContext) -> Self {
        Self {
            solver: Solver::new_almanac_frame(
                &cfg,
                ctx.apriori(),
                ctx.almanac,
                ctx.earth_cef,
                orbits: ctx.orbits_iter(),
            ),
            cggtts: true,
        }
    }
}