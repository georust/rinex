use rinex::{
    prelude::{Epoch, SV},
    navigation::Ephemeris,
};

use crate::context::QcContext;

use std::collections::HashMap;

pub struct EphemerisContext<'a> {
    eos: bool,
    toc: Epoch,
    sv: SV,
    buffer: HashMap<SV, Vec<(Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, &'a Epoch, &'a Ephemeris)> + 'a>,
}

impl QcContext {
    
    pub fn ephemeris_context(&self) -> Option<EphemerisContext> {

        if !self.has_navigation_data() {
            return None;
        }

        if let Some(brdc) = ctx.data.brdc_navigation_data() {
            info!("Ephemeris data source created.");
            let mut s = Self {
                eos: false,
                toc: Epoch::default(),
                sv: SV::default(),
                buffer: HashMap::with_capacity(32),
                iter: Box::new(brdc.ephemeris().map(|(toc, (_, sv, eph))| (sv, toc, eph))),
            };
            s.consume_many(32); // fill in with some data
            s
        } else {
            warn!("Operating without ephemeris source.");
            Self {
                eos: true,
                toc: Epoch::default(),
                sv: SV::default(),
                buffer: Default::default(),
                iter: Box::new([].into_iter()),
            }
        }
    }
    fn consume_one(&mut self) {
        if let Some((sv, toc, eph)) = self.iter.next() {
            if let Some(buffer) = self.buffer.get_mut(&sv) {
                buffer.push((*toc, eph.clone()));
            } else {
                self.buffer.insert(sv, vec![(*toc, eph.clone())]);
            }
            self.sv = sv;
            self.toc = *toc;
        } else {
            if !self.eos {
                info!("{}({}): consumed all epochs", self.toc, self.sv);
            }
            self.eos = true;
        }
    }
    fn consume_many(&mut self, n: usize) {
        for _ in 0..n {
            self.consume_one();
        }
    }
    fn try_select(&self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, &Ephemeris)> {
        let buffer = self.buffer.get(&sv)?;
        let sv_ts = sv.constellation.timescale()?;
        if sv.constellation.is_sbas() {
            buffer
                .iter()
                .filter_map(|(toc_i, eph_i)| {
                    if t >= *toc_i {
                        Some((*toc_i, *toc_i, eph_i))
                    } else {
                        None
                    }
                })
                .min_by_key(|(toc_i, _, _)| (t - *toc_i).abs())
        } else {
            buffer
                .iter()
                .filter_map(|(toc_i, eph_i)| {
                    if eph_i.is_valid(sv, t) && t >= *toc_i {
                        let toe_i = eph_i.toe(sv_ts)?;
                        Some((*toc_i, toe_i, eph_i))
                    } else {
                        None
                    }
                })
                .min_by_key(|(toc_i, _, _)| (t - *toc_i).abs())
        }
    }
    pub fn select(&mut self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, Ephemeris)> {
        let mut attempt = 0;
        loop {
            if let Some((toc_i, toe_i, eph_i)) = self.try_select(t, sv) {
                return Some((toc_i, toe_i, eph_i.clone()));
            } else {
                self.consume_one();
                attempt += 1;
            }
            if attempt == 10 {
                return None;
            }
        }
    }
}
