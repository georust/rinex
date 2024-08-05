use crate::{
    cli::Context,
    positioning::{Buffer, EphemerisSource},
};

use std::{cell::RefCell, collections::HashMap};

use gnss_rtk::prelude::{
    Almanac, Epoch, OrbitalState, OrbitalStateProvider, Vector3, EARTH_J2000, SUN_J2000, SV,
};
use rinex::prelude::Carrier;

use anise::errors::AlmanacError;

pub struct Orbit<'a, 'b> {
    eos: bool,
    has_precise: bool,
    eph: &'a RefCell<EphemerisSource<'b>>,
    buff: HashMap<SV, Buffer<(f64, f64, f64)>>,
    iter: Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + 'a>,
}

fn sun_unit_vector(almanac: &Almanac, t: Epoch) -> Result<Vector3<f64>, AlmanacError> {
    let earth_sun = almanac.transform(EARTH_J2000, SUN_J2000, t, None)?;
    Ok(Vector3::new(
        earth_sun.radius_km.x * 1000.0,
        earth_sun.radius_km.y * 1000.0,
        earth_sun.radius_km.z * 1000.0,
    ))
}

impl<'a, 'b> Orbit<'a, 'b> {
    pub fn new(ctx: &'a Context, eph: &'a RefCell<EphemerisSource<'b>>) -> Self {
        let has_precise = ctx.data.sp3().is_some();
        let mut s = Self {
            eph,
            has_precise,
            eos: if has_precise { false } else { true },
            buff: HashMap::with_capacity(16),
            iter: {
                if let Some(sp3) = ctx.data.sp3() {
                    if let Some(atx) = ctx.data.antex() {
                        info!("Orbit source created: operating with Ultra Precise Orbits.");
                        Box::new(sp3.sv_position().filter_map(|(t, sv, (x_km, y_km, z_km))| {
                            // TODO: needs rework and support all frequencies
                            let delta = atx.sv_antenna_apc_offset(t, sv, Carrier::L1)?;
                            let delta = Vector3::new(delta.0, delta.1, delta.2);
                            let r_sat = Vector3::new(x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3);
                            let k = -r_sat
                                / (r_sat[0].powi(2) + r_sat[1].powi(2) + r_sat[3].powi(2)).sqrt();

                            let r_sun = sun_unit_vector(&ctx.data.almanac, t).ok()?;
                            let norm = ((r_sun[0] - r_sat[0]).powi(2)
                                + (r_sun[1] - r_sat[1]).powi(2)
                                + (r_sun[2] - r_sat[2]).powi(2))
                            .sqrt();

                            let e = (r_sun - r_sat) / norm;
                            let j = Vector3::new(k[0] * e[0], k[1] * e[1], k[2] * e[2]);
                            let i = Vector3::new(j[0] * k[0], j[1] * k[1], j[2] * k[2]);
                            let r_dot = Vector3::new(
                                (i[0] + j[0] + k[0]) * delta[0],
                                (i[1] + j[1] + k[1]) * delta[1],
                                (i[2] + j[2] + k[2]) * delta[2],
                            );

                            let r_sat = r_sat - r_dot;
                            Some((t, sv, (r_sat[0], r_sat[1], r_sat[2])))
                        }))
                    } else {
                        info!("Orbit source created: operating with Precise Orbits.");
                        warn!("Cannot determine exact precise coordinates without ANTEX data: expect tiny errors (<1m).");
                        Box::new(sp3.sv_position())
                    }
                } else {
                    warn!("Orbit source created: operating without precise Orbits.");
                    Box::new([].into_iter())
                }
            },
        };
        if s.has_precise {
            s.consume_many(128); // fill in with some data
        }
        s
    }
    fn consume_one(&mut self) {
        if let Some((t, sv, (x_km, y_km, z_km))) = self.iter.next() {
            if let Some(buf) = self.buff.get_mut(&sv) {
                buf.push(t, (x_km, y_km, z_km));
            } else {
                let mut buf = Buffer::<(f64, f64, f64)>::new(31);
                buf.push(t, (x_km, y_km, z_km));
                self.buff.insert(sv, buf);
            }
        } else {
            if !self.eos {
                info!("Consumed all precise coordinates.");
            }
            self.eos = true;
        }
    }
    fn consume_many(&mut self, n: usize) {
        for _ in 0..n {
            self.consume_one();
        }
    }
}

impl OrbitalStateProvider for Orbit<'_, '_> {
    fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<OrbitalState> {
        let precise = if self.has_precise {
            // interpolation attempt
            if let Some(buffer) = self.buff.get_mut(&sv) {
                if let Some((x_km, y_km, z_km)) = buffer.contains(&t) {
                    Some(OrbitalState::from_position((
                        x_km * 1000.0,
                        y_km * 1000.0,
                        z_km * 1000.0,
                    )))
                } else {
                    if buffer.feasible(t, order) {
                        Some(OrbitalState::from_position(buffer.interpolate(
                            t,
                            order,
                            |buf| {
                                let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
                                for i in 0..=order {
                                    let mut li = 1.0_f64;
                                    let (t_i, (x_i, y_i, z_i)) = buf[i];
                                    for j in 0..=order {
                                        let (t_j, _) = buf[j];

                                        assert_eq!(
                                            t.time_scale, t_i.time_scale,
                                            "invalid input timescale: check your input!"
                                        );
                                        assert_eq!(
                                            t_i.time_scale, t_j.time_scale,
                                            "inconsistant timescales: aborting on internal error!"
                                        );
                                        if j != i {
                                            li *= (t - t_j).to_seconds();
                                            li /= (t_i - t_j).to_seconds();
                                        }
                                    }
                                    polynomials.0 += x_i * li;
                                    polynomials.1 += y_i * li;
                                    polynomials.2 += z_i * li;
                                }
                                let (x_km, y_km, z_km) =
                                    (polynomials.0, polynomials.1, polynomials.2);
                                debug!(
                                    "{}({}) precise state (km ECEF): x={},y={},z={}",
                                    t, sv, x_km, y_km, z_km
                                );
                                (x_km * 1000.0, y_km * 1000.0, z_km * 1000.0)
                            },
                        )))
                    } else {
                        // not feasible
                        self.consume_many(3);
                        None
                    }
                }
            } else {
                // create new buff, push some symbols
                let mut buffer = Buffer::new(order);
                self.buff.insert(sv, buffer);
                self.consume_many(order + 2);
                None
            }
        } else {
            None
        }; //precise

        let keplerian = if let Some((toc, toe, eph)) = self.eph.borrow_mut().select(t, sv) {
            let (x_km, y_km, z_km) = eph.kepler2position(sv, toc, t)?;
            debug!(
                "{}({}) keplerian state (km ECEF): x={},y={},z={}",
                t, sv, x_km, y_km, z_km
            );
            Some(OrbitalState::from_position((
                x_km * 1000.0,
                y_km * 1000.0,
                z_km * 1000.0,
            )))
        } else {
            None
        };

        if let Some(precise) = precise {
            Some(precise)
        } else {
            keplerian
        }
    }
}
