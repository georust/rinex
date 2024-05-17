use crate::cli::Context;
use gnss_rtk::prelude::{Duration, Epoch, SV};

mod interp;
use interp::Interpolator;

//mod nav;
//use nav::Time as NAVTime;

pub enum Time<'a> {
    //NAV(NAVTime<'a>),
    Interp(Interpolator<'a>),
}

impl<'a> Time<'a> {
    /*
     * Time source
     *  1. Prefer CLK product
     *  2. Prefer SP3 product
     *  3. BRDC last option
     */
    pub fn from_ctx(ctx: &'a Context) -> Self {
        if let Some(clk) = ctx.data.clock() {
            let iter = Box::new(
                clk.precise_sv_clock()
                    .map(|(t, sv, _, prof)| (t, sv, prof.bias)),
            );
            Self::Interp(Interpolator::from_iter(iter))
        } else {
            //if ctx.data.sp3_has_clock() {
            //    let sp3 = ctx.data.sp3().unwrap();
            //    let iter = sp3.sv_clock();
            //    Self::Interp(Interpolator::from_iter(iter))
            //} else {
            // TODO
            // let brdc = ctx.data.brdc_navigation().unwrap(); // infaillible
            // Box::new(brdc.sv_clock())
            // dt = t - toc
            // for (i=0; i<2; i++)
            //    dt -= a0 + a1 * dt+ a2 * dt^2
            // return a0 + a1 * dt + a2 * dt
            // let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);
            panic!("not supported yet");
            //}
        }
    }
    pub fn next_at(&mut self, t: Epoch, sv: SV) -> Option<Duration> {
        match self {
            Self::Interp(interp) => interp.next_at(t, sv),
        }
    }
}
