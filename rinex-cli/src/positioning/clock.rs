use crate::{cli::Context, positioning::EphemerisSelector};

use std::collections::HashMap;

use gnss_rtk::prelude::{Duration, Epoch, TimeScale, SV};

pub trait ClockStateProvider {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<Duration>;
}

pub struct Clock<E: EphemerisSelector> {
    eph: E,
}

impl<E: EphemerisSelector> ClockStateProvider for Clock<E> {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        // test if exist in buffer
        let (toc, eph) = self.eph.select(t, sv)?;
        let ts = sv.constellation.timescale()?;
        let toe = eph.toe(ts)?;
        let t_gpst = t.to_time_scale(TimeScale::GPST);
        let toc_gpst = toc.to_time_scale(TimeScale::GPST);
        let toe_gpst = toe.to_time_scale(TimeScale::GPST);
        let mut dt = (t_gpst - toc_gpst).to_seconds();
        let (a0, a1) = (eph.clock_bias, eph.clock_drift);
        for _ in 0..=2 {
            dt -= a0 + a1 * dt;
        }
        let dt = Duration::from_seconds(a0 + a1 * dt);
        debug!("{}({}) estimated clock correction: {}", t, sv, dt);
        Some(dt)
    }
}

impl<E: EphemerisSelector> Clock<E> {
    pub fn new(eph: E) -> Self {
        info!("Clock source created");
        Self { eph }
        //if let Some(clk) = ctx.data.clock() {
        //    let iter = Box::new(
        //        clk.precise_sv_clock()
        //            .map(|(t, sv, _, prof)| (t, sv, prof.bias)),
        //    );
        //    Self::Interp(Interpolator::from_iter(iter))
        //} else {
        //    if ctx.data.sp3_has_clock() {
        //        let sp3 = ctx.data.sp3().unwrap();
        //        let iter = sp3.sv_clock();
        //        Self::Interp(Interpolator::from_iter(iter))
        //    } else {
        //        let brdc = ctx.data.brdc_navigation().unwrap(); // infaillible
        //        let iter = brdc.ephemeris().map(|(toc, (_, sv, eph))| (sv, toc, eph));
        //        Self::NAV(NAVTime::from_iter(iter))
        //    }
        //}
    }
}
