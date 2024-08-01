use crate::cli::Context;
use std::collections::HashMap;

use gnss_rtk::prelude::{Epoch, OrbitalState, OrbitalStateProvider, TimeScale, SV};

use rinex::navigation::Ephemeris;

pub struct Orbit<'a> {
    buffer: HashMap<SV, Vec<(Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, &'a Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        let brdc = ctx
            .data
            .brdc_navigation()
            .expect("BRDC navigation required");
        Self {
            buffer: HashMap::with_capacity(64),
            iter: Box::new(brdc.ephemeris().map(|(toc, (_, sv, eph))| (sv, toc, eph))),
        }
    }
    fn feasible(&self, t: Epoch, sv: SV, sv_ts: TimeScale) -> bool {
        if sv.constellation.is_sbas() {
            // TOE does not exist
            self.buffer.get(&sv).is_some()
        } else {
            let max_dtoe = Ephemeris::max_dtoe(sv.constellation).unwrap();
            if let Some(dataset) = self.buffer.get(&sv) {
                let mut index = dataset.len();
                while index > 1 {
                    index -= 1;
                    let eph_i = &dataset[index].1;
                    if let Some(toe) = eph_i.toe_gpst(sv_ts) {
                        if toe < t && (t - toe) < max_dtoe {
                            return true;
                        }
                    }
                }
            }
            false
        }
    }
}

impl OrbitalStateProvider for Orbit<'_> {
    fn next_at(&mut self, t: Epoch, sv: SV, _: usize) -> Option<OrbitalState> {
        let sv_ts = sv.timescale()?;

        while !self.feasible(t, sv, sv_ts) {
            if let Some((sv_i, toc_i, eph_i)) = self.iter.next() {
                if let Some(dataset) = self.buffer.get_mut(&sv_i) {
                    dataset.push((*toc_i, eph_i.clone()));
                } else {
                    self.buffer.insert(sv_i, vec![(*toc_i, eph_i.clone())]);
                }
            } else {
                // EOF
                return None;
            }
        }

        let output = match self.buffer.get(&sv) {
            Some(eph) => {
                if sv.constellation.is_sbas() {
                    let (_toc_i, eph_i) = eph.iter().filter(|(toc_i, _)| *toc_i < t).min_by_key(
                        |(_toc_i, eph_i)| {
                            let toe_i = eph_i.toe_gpst(sv_ts).unwrap();
                            t - toe_i
                        },
                    )?;

                    let (x, y, z) = (
                        eph_i.get_orbit_f64("satPosX")? * 1.0E3,
                        eph_i.get_orbit_f64("satPosY")? * 1.0E3,
                        eph_i.get_orbit_f64("satPosZ")? * 1.0E3,
                    );
                    // NAV RINEX null payload means missing field
                    if x == 0.0 || y == 0.0 || z == 0.0 {
                        return None;
                    }
                    //let (vx_kms, vy_kms, vz_kms) = (
                    //    eph_i.get_orbit_f64("velX")? * 1.0E3,
                    //    eph_i.get_orbit_f64("velY")? * 1.0E3,
                    //    eph_i.get_orbit_f64("velZ")? * 1.0E3,
                    //);
                    //let (ax_kms, ay_kms, az_kms) = (
                    //    eph_i.get_orbit_f64("accelX")? * 1.0E3,
                    //    eph_i.get_orbit_f64("accelY")? * 1.0E3,
                    //    eph_i.get_orbit_f64("accelZ")? * 1.0E3,
                    //);
                    //let (x, y, z) = (
                    //    x
                    //        + vx_kms * dt,
                    //        //+ ax_kms * dt * dt / 2.0,
                    //    y
                    //        + vy_kms * dt,
                    //        //+ ay_kms * dt * dt / 2.0,
                    //    z
                    //        + vz_kms * dt,
                    //        //+ az_kms * dt * dt / 2.0,
                    //);
                    Some(OrbitalState::from_position((x, y, z)))
                } else {
                    let (_, eph_i) = eph.iter().filter(|(toc_i, _)| *toc_i < t).min_by_key(
                        |(_toc_i, eph_i)| {
                            let toe_i = eph_i.toe_gpst(sv_ts).unwrap();
                            t - toe_i
                        },
                    )?;

                    let (x_km, y_km, z_km) = eph_i.kepler2position(sv, t)?;
                    let (x, y, z) = (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3);
                    Some(OrbitalState::from_position((x, y, z)))
                }
            },
            None => None,
        };
        // TODO improve memory footprint: avoid memory growth
        //self.buffer.retain(|sv_i, ephemeris| {
        //    if *sv_i == sv {
        //        let max_dtoe = Ephemeris::max_dtoe(sv.constellation)
        //            .unwrap()
        //            .to_seconds();
        //        ephemeris.retain(|eph_i| {
        //            let toe = eph_i.toe_gpst(sv_ts).unwrap();
        //            let dt = (t - toe).to_seconds();
        //            if dt < max_dtoe {
        //                dt > 0.0
        //            } else {
        //                false
        //            }
        //        });
        //        !ephemeris.is_empty()
        //    } else {
        //        true
        //    }
        //});
        output
    }
}
