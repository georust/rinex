use crate::{
    cli::Context,
    positioning::{Buffer, EphemerisSource},
};

use std::{cell::RefCell, collections::HashMap};

use gnss_rtk::prelude::{ClockCorrection, Duration, Epoch, SV};

pub trait ClockStateProvider {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<ClockCorrection>;
}

pub struct Clock<'a, 'b> {
    eos: bool,
    has_precise: bool,
    buff: HashMap<SV, Buffer<f64>>,
    eph: &'a RefCell<EphemerisSource<'b>>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a>,
}

impl ClockStateProvider for Clock<'_, '_> {
    fn next_clock_at(&mut self, t: Epoch, sv: SV) -> Option<ClockCorrection> {
        if self.has_precise {
            // interpolation attempt
            if let Some(buffer) = self.buff.get_mut(&sv) {
                if let Some(dt) = buffer.contains(&t) {
                    let dt = Duration::from_seconds(*dt);
                    return Some(ClockCorrection::without_relativistic_correction(dt));
                } else {
                    if buffer.feasible(t, 2) {
                        let dt = buffer.interpolate(t, 2, |buf| {
                            let (t_0, dt_0) = buf[0];
                            let (t_1, dt_1) = buf[1];
                            let delta_s = (t_1 - t_0).to_seconds();
                            let mut dt = (t_1 - t).to_seconds() / delta_s * dt_0;
                            dt += (t - t_0).to_seconds() / delta_s * dt_1;
                            dt
                        });
                        let dt = Duration::from_seconds(dt);
                        debug!("{}({}) precise correction {}", t, sv, dt);
                        return Some(ClockCorrection::without_relativistic_correction(dt));
                    } else {
                        self.consume_many(3);
                    }
                }
            } else {
                // create new buffer, push some symbols
                let buffer = Buffer::new(2);
                self.buff.insert(sv, buffer);
                self.consume_many(3);
            }
        }
        let (toc, _, eph) = self.eph.borrow_mut().select(t, sv)?;
        let dt = eph.clock_correction(toc, t, sv, 8)?;
        debug!("{}({}) estimated clock correction: {}", t, sv, dt);
        Some(ClockCorrection::without_relativistic_correction(dt))
    }
}

impl<'a, 'b> Clock<'a, 'b> {
    pub fn new(ctx: &'a Context, eph: &'a RefCell<EphemerisSource<'b>>) -> Self {
        let has_precise = ctx.data.clock_data().is_some();
        let mut s = Self {
            eph,
            has_precise,
            eos: if has_precise { false } else { true },
            buff: HashMap::with_capacity(128),
            iter: if let Some(clk) = ctx.data.clock_data() {
                info!("Clock source created: operating with Precise Clock.");
                Box::new(
                    clk.precise_sv_clock()
                        .map(|(t, sv, _, prof)| (t, sv, prof.bias)),
                )
            } else {
                warn!("Clock source created: operating without Precise Clock.");
                Box::new([].into_iter())
            },
        };
        if has_precise {
            s.consume_many(128); // fill in with some data
        }
        s
    }
    fn consume_many(&mut self, n: usize) {
        for _ in 0..n {
            self.consume_one();
        }
    }
    fn consume_one(&mut self) {
        if let Some((t, sv, dt)) = self.iter.next() {
            if let Some(buf) = self.buff.get_mut(&sv) {
                buf.push(t, dt);
            } else {
                let mut buf = Buffer::<f64>::new(8);
                buf.push(t, dt);
                self.buff.insert(sv, buf);
            }
        } else {
            if !self.eos {
                info!("Consumed all precise clocks.");
            }
            self.eos = true;
        }
    }
}
