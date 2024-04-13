use crate::cli::Context;
use gnss_rtk::prelude::{
    AprioriPosition, Epoch, InterpolationResult as RTKInterpolationResult, SV,
};
use std::collections::HashMap;

use super::Buffer as BufferTrait;

struct Buffer {
    pub order: usize,
    pub inner: Vec<Epoch, f64>,
}

impl BufferTrait<f64> for Buffer {
    fn malloc(order: usize) -> Self {
        if order % 2 == 0 {
            panic!("only odd interpolation orders currently supported");
        }
        Self {
            order,
            inner: Vec::with_capacity(order),
        }
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn at_capacity(&self) -> bool {
        self.inner.len() == self.order
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn get(&self, idx: usize) -> Option<&(Epoch, f64))> {
        self.inner.get(idx)
    }
    fn last(&self) -> Option<&(Epoch, f64)> {
        self.inner.last()
    }
    fn push(&mut self, x_j: (Epoch, f64)) {
        self.inner.push(x_j);
    }
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    order: usize,
    ptr: Option<Epoch>,
    apriori: AprioriPosition,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a>,
}

impl<'a> Interpolator<'a> {
    pub fn from_ctx(ctx: &'a Context, order: usize, apriori: AprioriPosition) -> Self {
        Self {
            order,
            apriori,
            ptr: None,
            buffers: HashMap::with_capacity(4),
            iter: if let Some(sp3) = ctx.data.sp3() {
                Box::new(
                    sp3.sv_position()
                        .map(|(t, sv, (x, y, z))| (t, sv, (x * 1.0E3, y * 1.0E3, z * 1.0E3))),
                )
            } else {
                panic!("only sp3 supported at the moment");
            },
        }
    }
    fn push_data(&mut self, t: Epoch, sv: SV, pos: (f64, f64, f64)) {
        if let Some(buf) = self.buffers.get_mut(&sv) {
            buf.fill((t, pos));
        } else {
            let mut buf = Buffer::malloc(self.order);
            buf.fill((t, pos));
            self.buffers.insert(sv, buf);
        }
        self.ptr = Some(t);
    }
    fn consume(&mut self, target: Epoch) {
        // consume one epoch completely
        let mut current = Option::<Epoch>::None;
        loop {
            if let Some(curr) = current {
                if let Some((t, sv, pos)) = self.iter.next() {
                    debug!("{:?} ({}) - filling buffer", t, sv);
                    self.push_data(t, sv, pos);
                    if t > curr {
                        // new Epoch
                        break;
                    } else {
                        current = Some(t);
                    }
                } else {
                    // consumed all data
                    break;
                }
            } else {
                if let Some((t, sv, pos)) = self.iter.next() {
                    debug!("{:?} ({}) - filling buffer", t, sv);
                    self.push_data(t, sv, pos);
                    current = Some(t);
                } else {
                    // consumed all data
                    break;
                }
            }
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<RTKInterpolationResult> {
        // Consume, until we get at least up to date
        loop {
            if self.ptr < Some(t) {
                debug!("{:?} - ({}) - needs more data", t, sv);
                self.consume(t);
            } else {
                break;
            }
        }
        // Consume until internal window is adjusted
        loop {
            if let Some(buf) = self.buffers.get_mut(&sv) {
                if buf.at_capacity() {
                    if let Some((_, dt)) = buf.dt() {
                        if let Some(midpoint) = buf.midpoint() {
                            if (midpoint - t).abs() <= dt {
                                debug!("{:?} - ({}) - centered", t, sv);
                                break;
                            } else {
                                // not centered
                                debug!("{:?} - ({}) - needs more data", t, sv);
                                self.consume(t);
                            }
                        } else {
                            debug!("{:?} - ({}) - needs more data", t, sv);
                            self.consume(t);
                        }
                    } else {
                        debug!("{:?} - ({}) - needs more data", t, sv);
                        self.consume(t);
                    }
                } else {
                    debug!("{:?} - ({}) - needs more data", t, sv);
                    self.consume(t);
                }
            } else {
                // non existing data: will never propose anything
                break;
            }
        }
        let buf = self.buffers.get(&sv)?;
        // interpolate (if need be)
        if let Some(pos) = buf
            .inner
            .iter()
            .filter_map(|(t_buf, pos)| {
                if *t_buf == t {
                    Some(pos)
                } else {
                    None
                }
            })
            .reduce(|k, _| k) 
        {
            Some(
                RTKInterpolationResult::from_apc_position(*pos) //TODO
                .with_elevation_azimuth((0.0_f64, 0.0_f64)), //TODO
            )
        } else {
            let midpoint = buf.inner[(self.order + 1) / 2];
            let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
            for i in 0..self.order {
                let mut li = 1.0_f64;
                let (t_i, (x_i, y_i, z_i)) = buf.inner[i];
                for j in 0..self.order {
                    let (t_j, (x_j, y_j, z_j)) = buf.inner[j];
                    if j != i {
                        li *= (t - t_i).to_seconds();
                        li /= (t_i - t_j).to_seconds();
                    }
                }
                polynomials.0 += x_i * li;
                polynomials.1 += y_i * li;
                polynomials.2 += z_i * li;
            }
            Some(
                RTKInterpolationResult::from_apc_position(polynomials) //TODO
                .with_elevation_azimuth((0.0_f64, 0.0_f64)), //TODO
            )
        }
    }
}
