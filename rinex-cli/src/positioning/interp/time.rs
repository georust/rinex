use crate::cli::Context;
use gnss_rtk::prelude::{Epoch, SV};
use std::collections::HashMap;

use super::Buffer as BufferTrait;

struct Buffer {
    inner: Vec<(Epoch, f64)>, //TODO
}

impl BufferTrait<f64> for Buffer {
    fn malloc(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    /// Fill internal Buffer, which does not tolerate data gaps
    fn at_capacity(&self) -> bool {
        self.inner.len() == 3
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn get(&self, idx: usize) -> Option<&(Epoch, f64)> {
        self.inner.get(idx)
    }
    fn latest(&self) -> Option<&(Epoch, f64)> {
        self.inner.last()
    }
    fn push(&mut self, x_j: (Epoch, f64)) {
        if self.len() == 3 {
            let mut index = 0;
            self.inner.retain(|r| {
                index += 1;
                index > 1
            });
        }
        self.inner.push(x_j);
    }
    fn snapshot(&self) -> &[(Epoch, f64)] {
        &self.inner
    }
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    ptr: Option<Epoch>,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a>,
}

impl<'a> Interpolator<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        Self {
            ptr: None,
            buffers: HashMap::with_capacity(4),
            iter: if let Some(clk) = ctx.data.clock() {
                Box::new(
                    clk.precise_sv_clock()
                        .map(|(t, sv, _, prof)| (t, sv, prof.bias)),
                )
            } else {
                panic!("clk data required at the moment");
            },
        }
    }
    fn push_data(&mut self, t: Epoch, sv: SV, dt: f64) {
        if let Some(buf) = self.buffers.get_mut(&sv) {
            buf.fill((t, dt));
        } else {
            let mut buf = Buffer::malloc(3);
            buf.fill((t, dt));
            self.buffers.insert(sv, buf);
        }
        self.ptr = Some(t);
    }
    // Consumses n epoch
    fn consume(&mut self, total: usize) {
        let mut cnt = 0;
        let mut current = Option::<Epoch>::None;
        loop {
            if let Some(curr) = current {
                if let Some((t, sv, pos)) = self.iter.next() {
                    self.push_data(t, sv, pos);
                    if t > curr {
                        debug!("{:?} ({}) - new epoch", t, sv);
                        cnt += 1;
                        if cnt == total {
                            break;
                        }
                    } else {
                        current = Some(t);
                    }
                } else {
                    info!("{:?} - consumed all data", curr);
                    break;
                }
            } else {
                if let Some((t, sv, pos)) = self.iter.next() {
                    self.push_data(t, sv, pos);
                    current = Some(t);
                } else {
                    info!("consumed all data");
                    break;
                }
            }
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<f64> {
        let mut needs_update = false;

        if self.buffers.get(&sv).is_none() {
            // Unknown target: consume data
            self.consume(3);
        }

        let buf = self.buffers.get(&sv)?;
        let snapshot = buf.snapshot();
        debug!("{:?} ({}) - {:?}", t, sv, snapshot);

        // special case: direct output
        if let Some(value) = snapshot
            .iter()
            .filter_map(|(t_buf, value)| if *t_buf == t { Some(value) } else { None })
            .reduce(|k, _| k)
        {
            return Some(*value);
        }

        // maintain centered buffer
        if let Some((t_0, _)) = snapshot.get(0) {
            needs_update |= t < *t_0;
        } else {
            needs_update |= true;
        }

        // maintain centered buffer
        if let Some((t_1, _)) = snapshot.get(2) {
            needs_update |= t > *t_1;
        } else {
            needs_update |= true;
        }

        let buf = self.buffers.get(&sv)?;
        let snapshot = buf.snapshot();

        if buf.at_capacity() {
            if let Some((x_0, y_0)) = snapshot.get(0) {
                if let Some((x_1, y_1)) = snapshot.get(2) {
                    debug!("{:?} ({}) - centered", t, sv);
                    let dx = (*x_1 - *x_0).to_seconds();
                    let mut dy = (*x_1 - t).to_seconds() / dx * y_0;
                    dy += (t - *x_0).to_seconds() / dx * y_1;
                    return Some(dy);
                }
            }
        }
        if needs_update {
            self.consume(1);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::positioning::interp::time::Buffer;
    use crate::positioning::interp::Buffer as BufferTrait;
    use rinex::prelude::{Duration, Epoch};
    use std::str::FromStr;
    #[test]
    fn buffer_fill() {
        let mut buf = Buffer::malloc(2);
        for t in ["2020-06-25T00:00:00 UTC", "2020-06-25T00:00:05 UTC"] {
            let t = Epoch::from_str(t).unwrap();
            buf.fill((t, 0.0_f64));
        }
        assert_eq!(
            Some((Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(), 0.0_f64,)),
            buf.latest().copied()
        );

        assert_eq!(
            buf.snapshot(),
            &[
                (Epoch::from_str("2020-06-25T00:00:00 UTC").unwrap(), 0.0_f64),
                (Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(), 0.0_f64),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(5.0));

        let t = Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap();
        buf.fill((t, 0.0_f64));

        assert_eq!(
            buf.snapshot(),
            &[
                (Epoch::from_str("2020-06-25T00:00:00 UTC").unwrap(), 0.0_f64),
                (Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(), 0.0_f64),
                (Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap(), 0.0_f64),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(5.0));

        let t = Epoch::from_str("2020-06-25T00:00:20 UTC").unwrap();
        buf.fill((t, 0.0_f64));

        assert_eq!(
            buf.snapshot(),
            &[(Epoch::from_str("2020-06-25T00:00:20 UTC").unwrap(), 0.0_f64),],
        )
    }
}
