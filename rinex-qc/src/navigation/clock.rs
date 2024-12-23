use crate::{context::QcContext, navigation::eph::EphemerisContext};

use std::cell::RefCell;

pub struct ClockContext<'a, 'b> {
    eph_ctx: &'a RefCell<EphemerisContext<'b>>,
}

impl<'a, 'b> ClockContext<'a, 'b> {
    pub fn new(eph_ctx: &'a RefCell<EphemerisContext<'b>>) -> Self {
        Self { eph_ctx }
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
