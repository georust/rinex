use crate::positioning::BufferTrait;
use gnss_rtk::prelude::{Duration, Epoch, SV};
use std::collections::HashMap;

#[derive(Debug)]
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
    fn feasible(&self, _order: usize, t: Epoch) -> bool {
        if self.len() < 2 {
            false
        } else {
            let t0 = self.get(self.len() - 1).unwrap().0; // latest
            let t1 = self.get(self.len() - 2).unwrap().0; // latest -1
            t1 <= t && t <= t0
        }
    }
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    /// Total counter
    epochs: usize,
    /// Internal buffer
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a>,
}

impl<'a> Interpolator<'a> {
    pub fn from_iter(iter: impl Iterator<Item = (Epoch, SV, f64)> + 'a) -> Self {
        Self {
            epochs: 0,
            iter: Box::new(iter),
            buffers: HashMap::with_capacity(32),
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
    // Returns true if interpolation is feasible @ t for SV
    fn is_feasible(&self, t: Epoch, sv: SV) -> bool {
        if let Some(buf) = self.buffers.get(&sv) {
            buf.feasible(1, t)
        } else {
            false
        }
    }
    // Clock offset interpolation @ t for SV
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        let mut dt = Option::<Duration>::None;
        let mut first_x = Option::<Epoch>::None;

        // Maintain buffer up to date, consume data if need be
        while !self.is_feasible(t, sv) {
            if self.consume(1) {
                // end of stream
                break;
            }
        }

        let buf = self.buffers.get_mut(&sv)?;

        if let Some(y) = buf.direct_output(t) {
            // No need to interpolate @ t for SV
            // Preserves data precision
            first_x = Some(t);
            dt = Some(Duration::from_seconds(*y));
        } else if let Some((before_x, before_y)) =
            buf.inner.iter().filter(|(v_t, _)| *v_t < t).last()
        {
            first_x = Some(*before_x);

            if let Some((after_x, after_y)) = buf
                .inner
                .iter()
                .filter(|(v_t, _)| *v_t >= t)
                .reduce(|k, _| k)
            {
                let dx = (*after_x - *before_x).to_seconds();
                let mut dy = (*after_x - t).to_seconds() / dx * *before_y;
                dy += (t - *before_x).to_seconds() / dx * *after_y;
                dt = Some(Duration::from_seconds(dy));
            }
        }
        // Discard symbols that did not contribute (too old)
        if let Some(first_x) = first_x {
            buf.inner.retain(|(k, _)| *k >= first_x);
        }
        dt
    }
}
