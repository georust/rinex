use gnss_rtk::prelude::{Duration, Epoch, TimeScale, SV};
use rinex::navigation::Ephemeris;
use std::collections::HashMap;

pub struct Time<'a> {
    buffer: HashMap<SV, Vec<(Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (&'a Epoch, SV, &'a Ephemeris)> + 'a>,
}

impl<'a> Time<'a> {
    pub fn from_iter(iter: impl Iterator<Item = (&'a Epoch, SV, &'a Ephemeris)> + 'a) -> Self {
        Self {
            iter: Box::new(iter),
            buffer: HashMap::with_capacity(32),
        }
    }
    fn feasible(&self, t: Epoch, sv: SV, sv_ts: TimeScale) -> bool {
        let max_dtoe = Ephemeris::max_dtoe(sv.constellation).unwrap();
        if let Some(dataset) = self.buffer.get(&sv) {
            let mut index = dataset.len();
            while index > 1 {
                index -= 1;
                let (_, eph_i) = &dataset[index];
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
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        let sv_ts = sv.timescale()?;

        while !self.feasible(t, sv, sv_ts) {
            if let Some((toc_i, sv_i, eph_i)) = self.iter.next() {
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
            None => None,
            Some(ephemeris) => {
                let (toc_i, eph_i) = ephemeris.iter().min_by_key(|(toc_i, eph_i)| {
                    let toe_i = eph_i.toe_gpst(sv_ts).unwrap();
                    t - toe_i
                    //(t - *toc_i).abs()
                })?;

                let (a0, a1, a2) = (eph_i.clock_bias, eph_i.clock_drift, eph_i.clock_drift_rate);

                let t_gpst = t.to_time_scale(TimeScale::GPST).duration.to_seconds();

                let toe_gpst = eph_i.toe_gpst(sv_ts)?.duration.to_seconds();

                let toc_gpst = toc_i.to_time_scale(TimeScale::GPST).duration.to_seconds();

                let mut dt = t_gpst - toc_gpst;

                if dt > 302400.0 {
                    dt -= 604800.0;
                } else if dt < -302400.0 {
                    dt += 604800.0;
                }

                let ts = dt;
                for _ in 0..=2 {
                    dt = ts - (a0 + a1 * dt + a2 * dt.powi(2));
                }
                Some(Duration::from_seconds(a0 + a1 * dt + a2 * dt.powi(2)))
            },
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
