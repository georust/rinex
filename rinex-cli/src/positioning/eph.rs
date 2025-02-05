use crate::cli::Context;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Epoch, SV};
use std::collections::HashMap;

pub struct EphemerisSource<'a> {
    eos: bool,
    toc: Epoch,
    sv: SV,
    buffer: HashMap<SV, Vec<(Epoch, Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, Epoch, Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> EphemerisSource<'a> {
    /// Builds new [EphemerisSource] from [Context]
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if let Some(brdc) = ctx.data.brdc_navigation() {
            info!("Ephemeris data source created.");
            let mut s = Self {
                eos: false,
                toc: Epoch::default(),
                sv: SV::default(),
                buffer: HashMap::with_capacity(32),
                iter: Box::new(brdc.nav_ephemeris_frames_iter().filter_map(|(k, v)| {
                    let sv_ts = k.sv.timescale()?;
                    let toe = v.toe(sv_ts)?;
                    Some((k.sv, k.epoch, toe, v))
                })),
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

    /// Consume one entry from [Iterator]
    fn consume_one(&mut self) {
        if let Some((sv, toc, toe, eph)) = self.iter.next() {
            if let Some(buffer) = self.buffer.get_mut(&sv) {
                buffer.push((toc, eph.clone()));
            } else {
                self.buffer.insert(sv, vec![(toc, toe, eph.clone())]);
            }
            self.sv = sv;
            self.toc = toc;
        } else {
            if !self.eos {
                info!("{}({}): consumed all epochs", self.toc, self.sv);
            }
            self.eos = true;
        }
    }

    /// Consume n entries from [Iterator]
    fn consume_many(&mut self, n: usize) {
        for _ in 0..n {
            self.consume_one();
        }
    }

    /// [Ephemeris] selection attempt, for [SV] at [Epoch]
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

    /// [Ephemeris] selection at [Epoch] for [SV].
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
