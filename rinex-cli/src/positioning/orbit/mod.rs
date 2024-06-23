use crate::cli::Context;
use gnss_rtk::prelude::{Epoch, InterpolationResult, SV};

//mod sp3;
//use sp3::Orbit as SP3Orbit;

mod nav;
use nav::Orbit as NAVOrbit;

pub enum Orbit<'a> {
    //SP3(SP3Orbit<'a>),
    NAV(NAVOrbit<'a>),
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context, order: usize) -> Self {
        //if ctx.data.has_sp3() {
        //    Self::SP3(SP3Orbit::from_ctx(ctx, order))
        //} else {
        Self::NAV(NAVOrbit::from_ctx(ctx))
        //}
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<InterpolationResult> {
        match self {
            //Self::SP3(orbit) => orbit.next_at(t, sv),
            Self::NAV(orbit) => orbit.next_at(t, sv),
        }
    }
}
