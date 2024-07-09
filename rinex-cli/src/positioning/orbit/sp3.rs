use crate::{cli::Context, positioning::BufferTrait};
use std::collections::HashMap;

use gnss_rtk::prelude::{
    Almanac, Epoch, InterpolationResult as RTKInterpolationResult, TimeScale, Vector3, EARTH_J2000,
    SPK, SUN_J2000, SV,
};

use rinex::carrier::Carrier;

use anise::almanac::metaload::MetaFile;

#[derive(Debug)]
struct Buffer {
    inner: Vec<(Epoch, (f64, f64, f64))>,
}

impl BufferTrait<(f64, f64, f64)> for Buffer {
    fn malloc(size: usize) -> Self {
        Self {
            inner: Vec::with_capacity(size),
        }
    }
    fn push(&mut self, x_j: (Epoch, (f64, f64, f64))) {
        self.inner.retain(|k| *k != x_j);
        self.inner.push(x_j);
    }
    fn clear(&mut self) {
        self.inner.clear();
    }
    fn snapshot(&self) -> &[(Epoch, (f64, f64, f64))] {
        &self.inner
    }
    fn snapshot_mut(&mut self) -> &mut [(Epoch, (f64, f64, f64))] {
        &mut self.inner
    }
    fn get(&self, index: usize) -> Option<&(Epoch, (f64, f64, f64))> {
        self.inner.get(index)
    }
    fn len(&self) -> usize {
        self.inner.len()
    }
    fn feasible(&self, order: usize, t: Epoch) -> bool {
        let before_t = self.inner.iter().filter(|(k, _)| *k < t).count();
        let after_t = self.inner.iter().filter(|(k, _)| *k > t).count();
        let size = (order + 1) / 2; // restricted to odd orders
        before_t >= size && after_t >= size
    }
}

/*
 * Evaluates Sun / Earth vector in meters ECEF at "t"
 */
fn sun_unit_vector(almanac: &Almanac, t: Epoch) -> Vector3<f64> {
    let earth2sun = almanac.transform(EARTH_J2000, SUN_J2000, t, None).unwrap();

    Vector3::new(
        earth2sun.radius_km.x * 1000.0,
        earth2sun.radius_km.y * 1000.0,
        earth2sun.radius_km.z * 1000.0,
    )
}

pub struct Orbit<'a> {
    epochs: usize,
    order: usize,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + 'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context, order: usize) -> Self {
        let meta = MetaFile {
            uri: "http://public-data.nyxspace.com/anise/v0.4/de440s.bsp".to_string(),
            crc32: Some(3072159656), // Specifying the CRC allows only downloading the data once.
        };
        let almanac = Almanac::default()
            .load_from_metafile(meta)
            .unwrap_or_else(|e| panic!("failed to build Almanac: {}", e));

        let sp3 = ctx.data.sp3().unwrap();
        Self {
            order,
            epochs: 0,
            buffers: HashMap::with_capacity(128),
            iter: if let Some(atx) = ctx.data.antex() {
                Box::new(sp3.sv_position().filter_map(move |(t, sv, (x, y, z))| {
                    // TODO: need to complexify the whole interface
                    //       to provide correct information with respect to frequency
                    if let Some(delta) = atx.sv_antenna_apc_offset(t, sv, Carrier::L1) {
                        let delta = Vector3::<f64>::new(delta.0, delta.1, delta.2);
                        let r_sat = Vector3::<f64>::new(x * 1.0E3, y * 1.0E3, z * 1.0E3);
                        let k = -r_sat
                            / (r_sat[0].powi(2) + r_sat[1].powi(2) + r_sat[3].powi(2)).sqrt();

                        let r_sun = sun_unit_vector(&almanac, t);
                        let norm = ((r_sun[0] - r_sat[0]).powi(2)
                            + (r_sun[1] - r_sat[1]).powi(2)
                            + (r_sun[2] - r_sat[2]).powi(2))
                        .sqrt();

                        let e = (r_sun - r_sat) / norm;
                        let j = Vector3::<f64>::new(k[0] * e[0], k[1] * e[1], k[2] * e[2]);
                        let i = Vector3::<f64>::new(j[0] * k[0], j[1] * k[1], j[2] * k[2]);
                        let r_dot = Vector3::<f64>::new(
                            (i[0] + j[0] + k[0]) * delta[0],
                            (i[1] + j[1] + k[1]) * delta[1],
                            (i[2] + j[2] + k[2]) * delta[2],
                        );

                        let r_sat = r_sat + r_dot;
                        Some((t, sv, (r_sat[0], r_sat[1], r_sat[1])))
                    } else {
                        error!("{:?} ({}) - failed to determine APC offset", t, sv);
                        None
                    }
                }))
            } else {
                warn!("Cannot determine exact APC coordinates without ANTEX data.");
                warn!("Expect tiny offsets in final results.");
                Box::new(
                    sp3.sv_position()
                        .map(|(t, sv, (x, y, z))| (t, sv, (x * 1.0E3, y * 1.0E3, z * 1.0E3))),
                )
            },
        }
    }
    fn is_feasible(&self, t: Epoch, sv: SV) -> bool {
        if let Some(buf) = self.buffers.get(&sv) {
            buf.feasible(self.order, t)
        } else {
            false
        }
    }
    fn push(&mut self, t: Epoch, sv: SV, data: (f64, f64, f64)) {
        if let Some(buf) = self.buffers.get_mut(&sv) {
            buf.fill((t, data));
        } else {
            let mut buf = Buffer::malloc(self.order + 1);
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
                    }
                }
                prev_t = Some(t);
            } else {
                return true;
            }
        }
        self.epochs += epochs;
        false
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<RTKInterpolationResult> {
        // Maintain buffer up to date, consume data if need be
        while !self.is_feasible(t, sv) {
            if self.consume(1) {
                // end of stream
                return None;
            }
        }

        let buf = self.buffers.get_mut(&sv)?;
        if let Some((x, y, z)) = buf.direct_output(t) {
            // No need to interpolate @ t for SV
            // Preserves data precision
            return Some(RTKInterpolationResult::from_position((*x, *y, *z)));
        }

        let mut mid_offset = 0;
        let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut out = Option::<RTKInterpolationResult>::None;

        for (index, (t_i, _)) in buf.inner.iter().enumerate() {
            if *t_i > t {
                break;
            }
            mid_offset = index;
        }

        let (min_before, min_after) = ((self.order + 1) / 2, (self.order + 1) / 2);

        if mid_offset >= min_before && buf.len() - mid_offset >= min_after {
            let offset = mid_offset - (self.order + 1) / 2;
            for i in 0..=self.order {
                let mut li = 1.0_f64;

                let (mut t_i, (x_i, y_i, z_i)) = buf.inner[offset + i];
                if t_i.time_scale != TimeScale::GPST {
                    t_i = Epoch::from_gpst_duration(t_i.to_gpst_duration());
                }

                for j in 0..=self.order {
                    let (mut t_j, _) = buf.inner[offset + j];
                    if t_j.time_scale != TimeScale::GPST {
                        t_j = Epoch::from_gpst_duration(t_j.to_gpst_duration());
                    }

                    if j != i {
                        li *= (t - t_j).to_seconds();
                        li /= (t_i - t_j).to_seconds();
                    }
                }
                polynomials.0 += x_i * li;
                polynomials.1 += y_i * li;
                polynomials.2 += z_i * li;
            }
            out = Some(RTKInterpolationResult::from_position(polynomials));
        }

        if out.is_some() {
            // TODO improve memory footprint and avoid memory growth
            //let index_min = mid_offset - (self.order + 1) / 2 - 2;
            //let mut index = 0;
            // buf.inner.retain(|_| {
            //     index += 1;
            //     index > index_min
            // });
        }
        out
    }
}

#[cfg(test)]
mod test {
    use super::{Buffer, BufferTrait};
    use hifitime::Epoch;
    use std::str::FromStr;
    #[test]
    fn buffer_gap() {
        let mut buffer = Buffer::malloc(4);
        for (t, value) in [
            ("2020-01-01T00:00:00 UTC", 0.0_f64),
            ("2020-01-01T00:01:00 UTC", 1.0_f64),
            ("2020-01-01T00:02:00 UTC", 2.0_f64),
        ] {
            let t = Epoch::from_str(t).unwrap();
            buffer.fill((t, (value, value, value)));
        }

        assert_eq!(
            buffer.snapshot(),
            &[
                (
                    Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                    (1.0_f64, 1.0_f64, 1.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:02:00 UTC").unwrap(),
                    (2.0_f64, 2.0_f64, 2.0_f64)
                ),
            ]
        );

        let t = Epoch::from_str("2020-01-01T00:03:00 UTC").unwrap();
        buffer.fill((t, (3.0_f64, 3.0_f64, 3.0_f64)));

        assert_eq!(
            buffer.snapshot(),
            &[
                (
                    Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                    (1.0_f64, 1.0_f64, 1.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:02:00 UTC").unwrap(),
                    (2.0_f64, 2.0_f64, 2.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:03:00 UTC").unwrap(),
                    (3.0_f64, 3.0_f64, 3.0_f64)
                ),
            ]
        );

        let t = Epoch::from_str("2020-01-01T00:04:00 UTC").unwrap();
        buffer.fill((t, (4.0_f64, 4.0_f64, 4.0_f64)));

        assert_eq!(
            buffer.snapshot(),
            &[
                (
                    Epoch::from_str("2020-01-01T00:00:00 UTC").unwrap(),
                    (0.0_f64, 0.0_f64, 0.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:01:00 UTC").unwrap(),
                    (1.0_f64, 1.0_f64, 1.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:02:00 UTC").unwrap(),
                    (2.0_f64, 2.0_f64, 2.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:03:00 UTC").unwrap(),
                    (3.0_f64, 3.0_f64, 3.0_f64)
                ),
                (
                    Epoch::from_str("2020-01-01T00:04:00 UTC").unwrap(),
                    (4.0_f64, 4.0_f64, 4.0_f64)
                ),
            ]
        );

        let t = Epoch::from_str("2020-01-01T00:06:00 UTC").unwrap();
        buffer.fill((t, (6.0_f64, 6.0_f64, 6.0_f64)));

        assert_eq!(
            buffer.snapshot(),
            &[(
                Epoch::from_str("2020-01-01T00:06:00 UTC").unwrap(),
                (6.0_f64, 6.0_f64, 6.0_f64)
            ),]
        );
    }
}
