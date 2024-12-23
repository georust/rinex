use rinex::{
    navigation::Ephemeris,
    prelude::{Epoch, SV},
};

use crate::context::QcContext;

use std::collections::HashMap;

use log::error;

pub struct EphemerisContext<'a> {
    eos: bool,
    buffer: HashMap<SV, Vec<(Epoch, Ephemeris)>>,
    iter: Box<dyn Iterator<Item = (SV, &'a Epoch, &'a Ephemeris)> + 'a>,
}

impl<'a> EphemerisContext<'a> {
    fn consume_until(&mut self, sel_t: Epoch, sel_sv: SV) {
        loop {
            if let Some((sv, toc, eph)) = self.iter.next() {
                // store
                if let Some(data) = self.buffer.get_mut(&sv) {
                    data.push((*toc, eph.clone()));
                } else {
                    // new
                    self.buffer.insert(sv, vec![(*toc, eph.clone())]);
                }

                // verify validity; but we only test for specific target
                // others are simply buffered
                if sel_sv == sv {
                    if !eph.is_valid(sv, sel_t) {
                        break;
                    }
                }
            } else {
                self.eos = true;
                break;
            }
        }
    }

    /// Select closest [Ephemeris] in buffer, computes ToE at the same time.  
    /// NB: ToE does not exist for SBAS, ToC is simply copied to maintain the API.
    fn closest_in_time(&self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, &Ephemeris)> {
        let buffer = self.buffer.get(&sv)?;
        let sv_ts = sv.constellation.timescale()?;

        if sv.constellation.is_sbas() {
            buffer
                .iter()
                .map(|(toc_i, eph_i)| (*toc_i, *toc_i, eph_i))
                .min_by_key(|(toc_i, _, _)| (t - *toc_i).abs())
        } else {
            buffer
                .iter()
                .filter_map(|(toc_i, eph_i)| {
                    if eph_i.is_valid(sv, t) {
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
    /// ## Input
    /// - t: desired [Epoch]
    /// - sv: desired [SV]
    /// ## Output
    /// - toc: [Epoch]
    /// - toe: [Epoch]
    /// - eph: [Ephemeris]
    pub fn select(&mut self, t: Epoch, sv: SV) -> Option<(Epoch, Epoch, Ephemeris)> {
        if !self.eos {
            self.consume_until(t, sv);
        }

        let (toc, toe, eph) = self.closest_in_time(t, sv)?;
        Some((toc, toe, eph.clone()))
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
            buffer: HashMap::with_capacity(8),
            iter: Box::new(
                nav_dataset
                    .ephemeris()
                    .map(|(t, (_, sv, eph))| (sv, t, eph)),
            ),
        })
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};
    use rinex::prelude::{Epoch, SV};
    use std::str::FromStr;

    #[test]
    #[cfg(feature = "flate2")]
    fn test_ephemeris_buffer() {
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(format!(
            "{}/../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        ctx.load_gzip_file(format!(
            "{}/../test_resources/NAV/V3/MOJN00DNK_R_20201770000_01D_MN.rnx.gz",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();

        let mut ctx = ctx.ephemeris_context().expect("ephemeris context failure");

        for (t, sv, exists) in [(
            Epoch::from_str("2020-06-25T04:30:00 GPST").unwrap(),
            SV::from_str("G01").unwrap(),
            true,
        )] {
            if exists {
                let (toc, toe, eph) = ctx.select(t, sv).unwrap();
            }
        }
    }
}
