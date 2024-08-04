use crate::positioning::EphemerisSource;
use std::cell::RefCell;

use gnss_rtk::prelude::{Duration, Epoch, TimeScale, SV};

pub trait ClockStateProvider {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<Duration>;
}

pub struct Clock<'a, 'b> {
    eph: &'a RefCell<EphemerisSource<'b>>,
}

impl ClockStateProvider for Clock<'_, '_> {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        let (toc, _, eph) = self.eph.borrow_mut().select(t, sv)?;
        let dt = eph.clock_correction(toc, t, sv, 8)?;
        debug!("{}({}) estimated clock correction: {}", t, sv, dt);
        Some(dt)
    }
}

impl<'a, 'b> Clock<'a, 'b> {
    pub fn new(eph: &'a RefCell<EphemerisSource<'b>>) -> Self {
        info!("Clock source created.");
        Self { eph }
    }
}
