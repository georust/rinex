use crate::{
    cli::Context,
    positioning::{Buffer, EphemerisSource},
};

use std::cell::RefCell;
use std::collections::HashMap;

use gnss_rtk::prelude::{
    Almanac, Epoch, OrbitalState, OrbitalStateProvider, Vector3, EARTH_J2000, SUN_J2000, SV,
};
use rinex::prelude::Carrier;

use anise::errors::AlmanacError;

pub struct Orbit<'a, 'b> {
    eos: bool,
    eph: &'a RefCell<EphemerisSource<'b>>,
    coords_buff: HashMap<SV, Buffer<(f64, f64, f64)>>,
    coords_iter: Box<dyn Iterator<Item = (Epoch, SV, (f64, f64, f64))> + 'a>,
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
        Self {
            eph,
            eos: false,
            coords_buff: HashMap::with_capacity(16),
            coords_iter: {
                if let Some(sp3) = ctx.data.sp3() {
                    info!("Operating with precise Orbits.");
                    if let Some(atx) = ctx.data.antex() {
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
                        warn!("Cannot determine exact APC state without ANTEX data.");
                        warn!("Expect tiny errors in final results");
                        Box::new(sp3.sv_position().map(|(t, sv, (x_km, y_km, z_km))| {
                            (t, sv, (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3))
                        }))
                    }
                } else {
                    warn!("Operating without precise Orbits.");
                    Box::new([].into_iter())
                }
            },
        }
    }
}

impl OrbitalStateProvider for Orbit<'_, '_> {
    fn next_at(&mut self, t: Epoch, sv: SV, order: usize) -> Option<OrbitalState> {
        let mut precise = Option::<OrbitalState>::None;
        let mut keplerian = Option::<OrbitalState>::None;

        // test if it exists in buffer
        if let Some((toc, toe, eph)) = self.eph.borrow_mut().select(t, sv) {
            let (x_km, y_km, z_km) = eph.kepler2position(sv, t)?;
            debug!(
                "{}({}) keplerian state (km ECEF): x={},y={},z={}",
                t, sv, x_km, y_km, z_km
            );
            keplerian = Some(OrbitalState::from_position((
                x_km * 1000.0,
                y_km * 1000.0,
                z_km * 1000.0,
            )));
        }

        // interpolation attempt
        if let Some(buffer) = self.coords_buff.get_mut(&sv) {
            if buffer.feasible(t) {
                precise = Some(OrbitalState::from_position(buffer.interpolate(t, |buf| {
                    let mut polynomials = (0.0_f64, 0.0_f64, 0.0_f64);
                    for i in 0..=order {
                        let mut li = 1.0_f64;
                        let (t_i, (x_i, y_i, z_i)) = buf[i];
                        for j in 0..=order {
                            let (t_j, _) = buf[j];

                            if t.time_scale != t_i.time_scale {
                                panic!("invalid input timescale");
                            }
                            if t_i.time_scale != t_j.time_scale {
                                panic!("epochs not expressed in same timescale");
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
                    let (x_km, y_km, z_km) = (
                        polynomials.0 * 1000.0,
                        polynomials.1 * 1000.0,
                        polynomials.2 * 1000.0,
                    );
                    debug!(
                        "{}({}) precise state (km ECEF): x={},y={},z={}",
                        t, sv, x_km, y_km, z_km
                    );
                    (x_km, y_km, z_km)
                })));
            } else {
                // not feasible
                // 1. clear past symbols
                buffer.discard(t, order);
                // 2. push new symbols
                if let Some((t, sv, coords)) = self.coords_iter.next() {
                    if let Some(buffer) = self.coords_buff.get_mut(&sv) {
                        buffer.push(t, coords);
                    } else {
                        let mut buffer = Buffer::new(order);
                        buffer.push(t, coords);
                        self.coords_buff.insert(sv, buffer);
                    }
                } else {
                    self.eos = true;
                    debug!("{}({}) consumed all precise states", t, sv);
                }
            }
        } else {
            // create new buff and push one symbol
            let mut buffer = Buffer::new(order);
            self.coords_buff.insert(sv, buffer);
        };

        if let Some(precise) = precise {
            Some(precise)
        } else {
            keplerian
        }
    }
}
