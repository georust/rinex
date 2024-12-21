use rinex::{
    navigation::Ephemeris,
    prelude::{Epoch, SV},
};

use crate::context::QcContext;

use std::collections::HashMap;

pub struct EphemerisContext<'a> {
    eos: bool,
    sv: SV,
    toc: Epoch,
    buffer: HashMap<SV, Vec<(Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, &'a Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> EphemerisContext<'a> {
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

    /// Select appropriate [Ephemeris] for navigation purposes
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

impl QcContext {
    pub fn ephemeris_context<'a>(&'a self) -> Option<EphemerisContext<'a>> {
        if !self.has_navigation_data() {
            return None;
        }

        let nav_dataset = self.nav_dataset.as_ref().unwrap();

        Some(EphemerisContext {
            eos: false,
            sv: Default::default(),
            toc: Default::default(),
            buffer: HashMap::with_capacity(8),
            iter: Box::new(
                nav_dataset
                    .ephemeris()
                    .map(|(t, (_, sv, eph))| (sv, t, eph)),
            ),
        })
    }
}
