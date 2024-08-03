use crate::cli::Context;
use gnss_rtk::prelude::{Almanac, Epoch, OrbitalState, OrbitalStateProvider, SV};

mod sp3;
use sp3::Orbit as SP3Orbit;

mod nav;
use nav::Orbit as NAVOrbit;

pub enum Orbit<'a> {
    SP3(SP3Orbit<'a>),
    NAV(NAVOrbit<'a>),
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context, order: usize, almanac: Almanac) -> Self {
        if ctx.data.has_sp3() {
            Self::SP3(SP3Orbit::from_ctx(ctx, order, almanac))
        } else {
            Self::NAV(NAVOrbit::from_ctx(ctx))
        }
    }
}

impl OrbitalStateProvider for Orbit<'_> {
    fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<OrbitalState> {
        match self {
            Self::SP3(orbit) => orbit.next_at(t, sv, order),
            Self::NAV(orbit) => orbit.next_at(t, sv, order),
        }
    }
}
