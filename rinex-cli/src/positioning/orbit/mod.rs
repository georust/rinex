use crate::cli::Context;

mod sp3;
use sp3::Orbit as SP3Orbit;

mod nav;
use nav::Orbit as NAVOrbit;

pub enum OrbitSource<'a> {
    SP3(SP3Orbit<'a>),
    NAV(NAVOrbit<'a>),
}

pub struct Orbit<'a> {
    source: OrbitSource<'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if ctx.has_sp3() {
            Self::SP3(SP3Orbit::from_ctx(ctx))
        } else {
            Self::NAV(NAVOrbit::from_ctx(ctx))
        }
    }
}
