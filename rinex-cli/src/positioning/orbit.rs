use crate::{
    positioning::EphemerisSource,
};

use std::cell::RefCell;

use gnss_rtk::prelude::{Epoch, OrbitalState, OrbitalStateProvider, SV};

pub struct Orbit<'a, 'b> {
    eph: &'a RefCell<EphemerisSource<'b>>,
}

impl<'a, 'b> Orbit<'a, 'b> {
    pub fn new(eph: &'a RefCell<EphemerisSource<'b>>) -> Self {
        info!("Orbit data source created");
        Self { eph }
    }
}

impl OrbitalStateProvider for Orbit<'_, '_> {
    fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<OrbitalState> {
        None
        //if let Some(next) = self.eph_iter.next_at(t, sv, order) {
        //    Some(next)
        //} else {
        //    self.coords_iter.next_at(t, sv, order)
        //}
    }
}
