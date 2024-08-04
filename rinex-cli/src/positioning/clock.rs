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
        // test if exists in buffer
        let (toc, toe, eph) = self.eph.borrow_mut().select(t, sv)?;
        let sv_ts = sv.constellation.timescale()?;
        let t_gpst = t.to_time_scale(TimeScale::GPST);
        let toc_gpst = toc.to_time_scale(TimeScale::GPST);
        let toe_gpst = toe.to_time_scale(TimeScale::GPST);
        let mut dt = (t_gpst - toe_gpst).to_seconds();
        let (a0, a1) = (eph.clock_bias, eph.clock_drift);
        for _ in 0..=2 {
            dt -= a0 + a1 * dt;
        }
        let dt = Duration::from_seconds(a0 + a1 * dt);
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
