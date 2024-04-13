use crate::cli::Context;
use gnss_rtk::prelude::{
    AprioriPosition, Epoch, InterpolationResult as RTKInterpolationResult, SV,
};
use std::collections::HashMap;

use super::Buffer as BufferTrait;

struct Buffer {
    order: usize,
    inner: Vec<(Epoch, (f64, f64, f64))>,
}

impl BufferTrait<(f64, f64, f64)> for Buffer {
    fn malloc(order: usize) -> Self {
        if order % 2 == 0 {
            panic!("only odd interpolation orders currently supported");
        }
        Self {
            order,
            inner: Vec::with_capacity(order + 1),
        }
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    /// Fill internal Buffer, which does not tolerate data gaps
    fn at_capacity(&self) -> bool {
        self.inner.len() == self.order + 1
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn get(&self, idx: usize) -> Option<&(Epoch, (f64, f64, f64))> {
        self.inner.get(idx)
    }
    fn latest(&self) -> Option<&(Epoch, (f64, f64, f64))> {
        self.inner.last()
    }
    fn push(&mut self, x_j: (Epoch, (f64, f64, f64))) {
        if self.len() == self.order + 1 {
            let mut index = 0;
            self.inner.retain(|r| {
                index += 1;
                index > 1
            });
        }
        self.inner.push(x_j);
    }
    fn snapshot(&self) -> &[(Epoch, (f64, f64, f64))] {
        &self.inner
    }
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    order: usize,
    ptr: Option<Epoch>,
    apriori: AprioriPosition,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + 'a>,
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
                panic!("sp3 data required at the moment");
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
    pub fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<RTKInterpolationResult> {
        let mut needs_update = false;

        if self.buffers.get(&sv).is_none() {
            // Unknown target: consume data
            self.consume(1);
            return None;
        }

        let buf = self.buffers.get(&sv)?;
        let snapshot = buf.snapshot();

        // special case: direct output
        if let Some(pos) = snapshot
            .iter()
            .filter_map(|(t_buf, pos)| if *t_buf == t { Some(pos) } else { None })
            .reduce(|k, _| k)
        {
            return Some(
                RTKInterpolationResult::from_apc_position(*pos) //TODO
                    .with_elevation_azimuth((0.0_f64, 0.0_f64)), //TODO
            );
        }

        // maintain windowed buffer
        if let Some((t_0, _)) = snapshot.get(snapshot.len() / 2 - 1) {
            if t < *t_0 {
                needs_update |= true;
            }
        }
        if let Some((t_1, _)) = snapshot.get(snapshot.len() / 2) {
            if t > *t_1 {
                needs_update |= true;
            }
        }

        let buf = self.buffers.get(&sv)?;
        let snapshot = buf.snapshot();

        if buf.at_capacity() && !needs_update {
            if let Some((t_0, _)) = snapshot.get(snapshot.len() / 2 - 1) {
                if let Some((t_1, _)) = snapshot.get(snapshot.len() / 2) {
                    if t > *t_0 && t < *t_1 {
                        // centered
                        debug!("{:?} ({}) - centered", t, sv);
                        let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
                        for i in 0..self.order + 1 {
                            let mut li = 1.0_f64;
                            let (t_i, (x_i, y_i, z_i)) = snapshot[i];
                            for j in 0..self.order + 1 {
                                let (t_j, _) = snapshot[j];
                                if j != i {
                                    li *= (t - t_j).to_seconds();
                                    li /= (t_i - t_j).to_seconds();
                                }
                            }
                            polynomials.0 += x_i * li;
                            polynomials.1 += y_i * li;
                            polynomials.2 += z_i * li;
                        }
                        return Some(
                            RTKInterpolationResult::from_apc_position(polynomials) //TODO
                                .with_elevation_azimuth((0.0_f64, 0.0_f64)), //TODO
                        );
                    } else {
                        needs_update |= true;
                    }
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
    use crate::positioning::interp::orbit::Buffer;
    use crate::positioning::interp::Buffer as BufferTrait;
    use rinex::prelude::{Duration, Epoch};
    use std::str::FromStr;
    #[test]
    fn buffer_fill() {
        let mut buf = Buffer::malloc(3);
        for t in [
            "2020-06-25T00:00:00 UTC",
            "2020-06-25T00:00:05 UTC",
            "2020-06-25T00:00:10 UTC",
        ] {
            let t = Epoch::from_str(t).unwrap();
            buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));
        }
        assert_eq!(
            Some((
                Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap(),
                (0.0_f64, 0.0_f64, 0.0_f64)
            )),
            buf.latest().copied()
        );

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:00 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(5.0));

        let t = Epoch::from_str("2020-06-25T00:00:15 UTC").unwrap();
        buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:00 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:15 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(5.0));

        let t = Epoch::from_str("2020-06-25T00:00:20 UTC").unwrap();
        buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:05 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:10 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:15 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:20 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(5.0));

        for t in [
            "2020-06-25T00:00:30 UTC",
            "2020-06-25T00:00:31 UTC",
            "2020-06-25T00:00:32 UTC",
        ] {
            let t = Epoch::from_str(t).unwrap();
            buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));
        }
        assert_eq!(
            Some((
                Epoch::from_str("2020-06-25T00:00:32 UTC").unwrap(),
                (0.0_f64, 0.0_f64, 0.0_f64)
            )),
            buf.latest().copied()
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(1.0));

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:30 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:31 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:32 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );
        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(1.0));

        let t = Epoch::from_str("2020-06-25T00:00:33 UTC").unwrap();
        buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:30 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:31 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:32 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:33 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );

        let (_, dt) = buf.dt().unwrap();
        assert_eq!(dt, Duration::from_seconds(1.0));

        let t = Epoch::from_str("2020-06-25T00:00:34 UTC").unwrap();
        buf.fill((t, (0.0_f64, 0.0_f64, 0.0_f64)));

        assert_eq!(
            buf.snapshot(),
            &[
                (
                    Epoch::from_str("2020-06-25T00:00:31 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:32 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:33 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-06-25T00:00:34 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
            ],
        );
    }
}
