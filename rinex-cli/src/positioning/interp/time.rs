use crate::cli::Context;
use gnss_rtk::prelude::{Duration, Epoch, SV};
use std::collections::HashMap;

/// Orbital state interpolator
pub struct Interpolator<'a> {
    epochs: usize,
    sampling: Duration,
    buffers: HashMap<SV, Vec<(Epoch, f64)>>,
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
            },
        }
    }
    fn push(&mut self, t: Epoch, sv: SV, data: f64) {
        if let Some(buf) = self.buffers.get_mut(&sv) {
            buf.push((t, data));
        } else {
            self.buffers.insert(sv, vec![(t, data)]);
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
                        //println!("{} - new epoch", t); // DEBUG
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
                    let last = v.iter().map(|(e, _)| e).last()?;
                    Some(last)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<f64> {
        // Maintain buffer up to date, consume data if need be
        loop {
            if let Some(latest) = self.latest(sv) {
                if *latest > t + self.sampling {
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

        let buf = self.buffers.get(&sv)?;
        if let Some((before_x, before_y)) = buf.iter().filter(|(v_t, _)| *v_t <= t).last() {
            // interpolate: if need be
            let dy: Option<f64> = if *before_x == t {
                Some(*before_y)
            } else {
                if let Some((after_x, after_y)) =
                    buf.iter().filter(|(v_t, _)| *v_t > t).reduce(|k, _| k)
                {
                    let dx = (*after_x - *before_x).to_seconds();
                    let mut dy = (*after_x - t).to_seconds() / dx * *before_y;
                    dy += (t - *before_x).to_seconds() / dx * *after_y;
                    Some(dy)
                } else {
                    None
                }
            };
            // management: discard old samples
            self.buffers.retain(|b_sv, b_v| {
                if *b_sv != sv {
                    true
                } else {
                    b_v.retain(|b_t| b_t.0 < t);
                    !b_v.is_empty()
                }
            });
            return dy;
        }
        None
    }
}
