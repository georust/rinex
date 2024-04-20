use super::Buffer as BufferTrait;
use crate::cli::Context;
use gnss_rtk::prelude::{Duration, Epoch, SV};
use std::collections::HashMap;

struct Buffer {
    inner: Vec<(Epoch, f64)>,
}

impl BufferTrait<f64> for Buffer {
    fn malloc(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }
    fn push(&mut self, x_j: (Epoch, f64)) {
        self.inner.push(x_j);
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn snapshot(&self) -> &[(Epoch, f64)] {
        &self.inner
    }
    fn snapshot_mut(&mut self) -> &mut [(Epoch, f64)] {
        &mut self.inner
    }
    fn get(&self, index: usize) -> Option<&(Epoch, f64)> {
        self.inner.get(index)
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    epochs: usize,
    sampling: Duration,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a>,
}

impl<'a> Interpolator<'a> {
    /*
     * Time interpolator
     *  1. Prefer CLK product
     *  2. Prefer SP3 product
     *  3. BRDC last option
     */
    pub fn from_ctx(ctx: &'a Context) -> Self {
        Self {
            epochs: 0,
            buffers: HashMap::with_capacity(32),
            // TODO improve sampling determination
            sampling: if ctx.data.clock().is_some() {
                Duration::from_seconds(30.0)
            } else {
                Duration::from_seconds(15.0 * 60.0)
            },
            iter: if let Some(clk) = ctx.data.clock() {
                Box::new(
                    clk.precise_sv_clock()
                        .map(|(t, sv, _, prof)| (t, sv, prof.bias)),
                )
            } else if let Some(sp3) = ctx.data.sp3() {
                Box::new(sp3.sv_clock())
            } else {
                panic!("sp3 or clock rinex currently required");
                // TODO
                // let brdc = ctx.data.brdc_navigation().unwrap(); // infaillible
                // Box::new(brdc.sv_clock())
                // dt = t - toc
                // for (i=0; i<2; i++)
                //    dt -= a0 + a1 * dt+ a2 * dt^2
                // return a0 + a1 * dt + a2 * dt
                // let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);
            },
        }
    }
    fn push(&mut self, t: Epoch, sv: SV, data: f64) {
        if let Some(buf) = self.buffers.get_mut(&sv) {
            buf.push((t, data));
        } else {
            let mut buf = Buffer::malloc(3);
            buf.fill((t, data));
            self.buffers.insert(sv, buf);
        }
    }
    // consumes N epochs completely
    fn consume(&mut self, total: usize) -> bool {
        let mut prev_t = None;
        let mut epochs = 0;
        while epochs < total {
            if let Some((t, sv, data)) = self.iter.next() {
                self.push(t, sv, data);
                if let Some(prev) = prev_t {
                    if t > prev {
                        epochs += 1;
                        // println!("{} - new epoch", t); // DEBUG
                    }
                }
                prev_t = Some(t);
            } else {
                //println!("consumed all data"); // DEBUG
                return true;
            }
        }
        self.epochs += epochs;
        false
    }
    fn latest(&self, sv: SV) -> Option<&Epoch> {
        self.buffers
            .iter()
            .filter_map(|(k, v)| {
                if *k == sv {
                    let last = v.inner.iter().map(|(e, _)| e).last()?;
                    Some(last)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        // Maintain buffer up to date, consume data if need be
        loop {
            if let Some(latest) = self.latest(sv) {
                if *latest >= t + self.sampling {
                    break;
                } else {
                    if self.consume(1) {
                        // end of stream
                        break;
                    }
                }
            } else {
                if self.consume(1) {
                    // end of stream
                    break;
                }
            }
        }

        let buf = self.buffers.get_mut(&sv)?;

        if let Some((before_x, before_y)) = buf.inner.iter().filter(|(v_t, _)| *v_t <= t).last() {
            // interpolate: if need be
            let dy: Option<Duration> = if *before_x == t {
                Some(Duration::from_seconds(*before_y))
            } else {
                if let Some((after_x, after_y)) = buf
                    .inner
                    .iter()
                    .filter(|(v_t, _)| *v_t > t)
                    .reduce(|k, _| k)
                {
                    let dx = (*after_x - *before_x).to_seconds();
                    let mut dy = (*after_x - t).to_seconds() / dx * *before_y;
                    dy += (t - *before_x).to_seconds() / dx * *after_y;
                    Some(Duration::from_seconds(dy))
                } else {
                    None
                }
            };

            // management: discard old samples
            if dy.is_some() {
                buf.inner.retain(|b| b.0 < t - self.sampling);
            }

            return dy;
        }
        None
    }
}
