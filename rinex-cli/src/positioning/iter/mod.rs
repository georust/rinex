use crate::cli::Context;
use gnss_rtk::prelude::{Epoch, InterpolationResult, SV};

mod eph;
use eph::EphemerisIter;

mod coords;
use coords::CoordsIter;

pub struct Iter<'a> {
    eph_iter: EphemerisIter<'a>,
    coords_iter: CoordsIter<'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context, order: usize) -> Self {
        Self {
            eph_iter: EphemerisIter::from_ctx(ctx),
            coords_iter: CoordsIter::from_ctx(ctx, order),
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<InterpolationResult> {
        if let Some(next) = self.coords_iter.next_at(t, sv) {
            Some(next)
        } else {
            self.eph_iter.next_at(t, sv)
        }
    }
}
