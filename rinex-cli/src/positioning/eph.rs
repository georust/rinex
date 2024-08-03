use crate::cli::Context;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Epoch, Rinex, SV};
use std::collections::HashMap;

pub struct EphemerisSource<'a> {
    latest: HashMap<SV, (Epoch, Ephemeris)>,
    iter: Box<dyn Iterator<Item = (SV, &'a Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> EphemerisSource<'a> {
    fn update(&mut self, next: (SV, &Epoch, &Ephemeris)) {
        let (sv, toc, eph) = next;
        self.latest.insert(sv, (*toc, eph.clone()));
    }
}

pub trait EphemerisSelector {
    fn select(&mut self, t: Epoch, sv: SV) -> Option<(Epoch, Ephemeris)>;
}

impl EphemerisSelector for EphemerisSource<'_> {
    fn select(&mut self, t: Epoch, sv: SV) -> Option<(Epoch, Ephemeris)> {
        if let Some((toc, latest)) = self.latest.get(&sv) {
            debug!("{}({}) - proposed latest toc={}", t, sv, toc);
            Some((*toc, latest.clone()))
        } else {
            if let Some(next) = self.iter.next() {
                self.update(next);
            }
            None
        }
    }
}

impl<'a> EphemerisSource<'a> {
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if let Some(brdc) = ctx.data.brdc_navigation() {
            info!("Ephemeris data source created.");
            Self {
                latest: HashMap::with_capacity(32),
                iter: Box::new(brdc.ephemeris().map(|(toc, (_, sv, eph))| (sv, toc, eph))),
            }
        } else {
            info!("Operating without ephemeris source.");
            Self {
                latest: Default::default(),
                iter: Box::new([].into_iter()),
            }
        }
    }
}
