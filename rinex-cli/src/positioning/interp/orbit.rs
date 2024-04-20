use super::Buffer as BufferTrait;
use crate::cli::Context;
use hifitime::Duration;
use std::collections::HashMap;

use gnss_rtk::prelude::{
    AprioriPosition, Arc, Bodies, Cosm, Epoch, Frame,
    InterpolationResult as RTKInterpolationResult, LightTimeCalc, Vector3, SV,
};

use rinex::{carrier::Carrier, navigation::Ephemeris};

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
}

/// Orbital state interpolator
pub struct Interpolator<'a> {
    order: usize,
    epochs: usize,
    sampling: Duration,
    apriori: AprioriPosition,
    buffers: HashMap<SV, Buffer>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + 'a>,
}

/*
 * Evaluates Sun / Earth vector in meters ECEF at "t"
 */
fn sun_unit_vector(ref_frame: &Frame, cosmic: &Arc<Cosm>, t: Epoch) -> Vector3<f64> {
    let sun_body = Bodies::Sun; // This only works in the solar system..
    let orbit = cosmic.celestial_state(sun_body.ephem_path(), t, *ref_frame, LightTimeCalc::None);
    Vector3::new(
        orbit.x_km * 1000.0,
        orbit.y_km * 1000.0,
        orbit.z_km * 1000.0,
    )
}

impl<'a> Interpolator<'a> {
    /*
     * Orbit interpolator
     *  1. Prefer SP3 product
     *  2. BRDC last option
     */
    pub fn from_ctx(ctx: &'a Context, order: usize, apriori: AprioriPosition) -> Self {
        let cosmic = Cosm::de438();
        let earth_frame = cosmic.frame("EME2000"); // this only works on planet Earth..
        assert!(
            order % 2 > 0,
            "only odd interpolation orders currently supported"
        );
        Self {
            order,
            apriori,
            epochs: 0,
            buffers: HashMap::with_capacity(128),
            sampling: Duration::from_seconds(15.0 * 60.0), // TODO improve this
            iter: if let Some(sp3) = ctx.data.sp3() {
                if let Some(atx) = ctx.data.antex() {
                    Box::new(sp3.sv_position().filter_map(move |(t, sv, (x, y, z))| {
                        // TODO: need to complexify the whole interface
                        //       to provide correct information with respect to frequency
                        if let Some(delta) = atx.sv_antenna_apc_offset(t, sv, Carrier::L1) {
                            let delta = Vector3::<f64>::new(delta.0, delta.1, delta.2);
                            let r_sat = Vector3::<f64>::new(x * 1.0E3, y * 1.0E3, z * 1.0E3);
                            let k = -r_sat
                                / (r_sat[0].powi(2) + r_sat[1].powi(2) + r_sat[3].powi(2)).sqrt();

                            let r_sun = sun_unit_vector(&earth_frame, &cosmic, t);
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
                }
            } else {
                unreachable!("sp3 data required at the moment");
            },
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
                        //println!("{:?} - new epoch", t); //DEBUG
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
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<RTKInterpolationResult> {
        // Maintain buffer up to date, consume data if need be
        let dt = Duration::from_seconds((self.order / 2) as f64 * self.sampling.to_seconds());
        loop {
            if let Some(latest) = self.latest(sv) {
                if *latest >= t + dt {
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

        // interp
        let buf = self.buffers.get_mut(&sv)?;
        //let len_before = buf.len(); // DEBUG
        let ecef = self.apriori.ecef;
        let mut mid_offset = 0;
        let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut out = Option::<RTKInterpolationResult>::None;

        for (index, (buf_t, buf_v)) in buf.inner.iter().enumerate() {
            if *buf_t == t {
                // special case: direct output
                let el_az = Ephemeris::elevation_azimuth(*buf_v, (ecef[0], ecef[1], ecef[2]));
                out = Some(
                    RTKInterpolationResult::from_apc_position(*buf_v).with_elevation_azimuth(el_az),
                );
                break;
            }
            if *buf_t > t {
                break;
            }
            mid_offset = index;
        }

        let (min_before, min_after) = ((self.order + 1) / 2, (self.order + 1) / 2);
        //println!("t: {:?} | offset [{}] | snapshot {:?}", t, mid_offset, buf); //DEBUG

        if out.is_none() {
            // needs interpolation
            if mid_offset >= min_before && buf.len() - mid_offset >= min_after {
                let offset = mid_offset - (self.order + 1) / 2;
                //println!("is feasible"); //DEBUG
                for i in 0..=self.order {
                    let mut li = 1.0_f64;
                    let (t_i, (x_i, y_i, z_i)) = buf.inner[offset + i];
                    for j in 0..=self.order {
                        let (t_j, _) = buf.inner[offset + j];
                        if j != i {
                            li *= (t - t_j).to_seconds();
                            li /= (t_i - t_j).to_seconds();
                        }
                    }
                    polynomials.0 += x_i * li;
                    polynomials.1 += y_i * li;
                    polynomials.2 += z_i * li;
                }

                let el_az = Ephemeris::elevation_azimuth(polynomials, (ecef[0], ecef[1], ecef[2]));
                out = Some(
                    RTKInterpolationResult::from_apc_position(polynomials)
                        .with_elevation_azimuth(el_az),
                );
                //} else {
                //    println!("not feasible"); //DEBUG
            }
        }

        if out.is_some() {
            // management: discard old samples
            let t_min = t - dt - self.sampling - self.sampling;
            buf.inner.retain(|b_t| b_t.0 > t_min);

            //let len_after = buf.len(); // DEBUG
            //if len_after != len_before { // DEBUG
            //    println!("{:?} - purge: t_min {:?} - snapshot {:?}", t, t_min, buf); //DEBUG
            //}
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
