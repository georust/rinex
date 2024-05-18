use crate::cli::Context;
use std::collections::HashMap;

use gnss_rtk::prelude::{
    AprioriPosition, Arc, Bodies, Cosm, Duration, Epoch, Frame,
    InterpolationResult as RTKInterpolationResult, LightTimeCalc, TimeScale, Vector3, SV,
};

use rinex::navigation::Ephemeris;

pub struct Orbit<'a> {
    apriori: AprioriPosition,
    buffer: HashMap<SV, Vec<Ephemeris>>,
    iter: Box<dyn Iterator<Item = (&'a Epoch, SV, &'a Ephemeris)> + 'a>,
}

impl<'a> Orbit<'a> {
    pub fn from_ctx(ctx: &'a Context, apriori: AprioriPosition) -> Self {
        let brdc = ctx
            .data
            .brdc_navigation()
            .expect("BRDC navigation required");
        Self {
            apriori,
            buffer: HashMap::with_capacity(64),
            iter: Box::new(brdc.ephemeris().map(|(toc, (_, sv, eph))| (toc, sv, eph))),
        }
    }
    fn feasible(&self, t: Epoch, sv: SV, sv_ts: TimeScale) -> bool {
        let max_dtoe = Ephemeris::max_dtoe(sv.constellation).unwrap();
        if let Some(dataset) = self.buffer.get(&sv) {
            let mut index = dataset.len();
            while index > 1 {
                index -= 1;
                let eph_i = &dataset[index];
                if let Some(toe) = eph_i.toe_gpst(sv_ts) {
                    if toe < t && (t - toe) < max_dtoe {
                        return true;
                    }
                }
            }
            false
        } else {
            false
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<RTKInterpolationResult> {
        let sv_ts = sv.timescale()?;

        while !self.feasible(t, sv, sv_ts) {
            if let Some((toc_i, sv_i, eph_i)) = self.iter.next() {
                if let Some(dataset) = self.buffer.get_mut(&sv_i) {
                    dataset.push(eph_i.clone());
                } else {
                    self.buffer.insert(sv_i, vec![eph_i.clone()]);
                }
            } else {
                // EOF
                return None;
            }
        }

        let ref_ecef = self.apriori.ecef();
        let output = match self.buffer.get(&sv) {
            Some(eph) => {
                let eph_i = eph.iter().min_by_key(|eph_i| {
                    let toe_i = eph_i.toe_gpst(sv_ts).unwrap();
                    t - toe_i
                })?;
                let (x_km, y_km, z_km) = eph_i.kepler2ecef(sv, t)?;
                let (x, y, z) = (x_km * 1.0E3, y_km * 1.0E3, z_km * 1.0E3);
                let el_az = Ephemeris::elevation_azimuth(
                    (x, y, z),
                    (ref_ecef[0], ref_ecef[1], ref_ecef[2]),
                );
                Some(RTKInterpolationResult::from_position((x, y, z)).with_elevation_azimuth(el_az))
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
