mod doris;
mod ionosphere;
mod navigation;
mod sp3_plot;

pub use doris::plot_doris_observations;
pub use navigation::plot_sv_nav_clock;
pub use navigation::plot_sv_nav_orbits;
pub use sp3_plot::plot_residual_ephemeris;

use crate::cli::Context;
use crate::graph::PlotContext;
use clap::ArgMatches;

use ionosphere::plot_ionospheric_delay;

pub fn plot_atmosphere_conditions(ctx: &Context, plot_ctx: &mut PlotContext, matches: &ArgMatches) {
    if matches.get_flag("tropo") {
        let _meteo = ctx.data.meteo().expect("--tropo requires METEO RINEX");
    }
    if matches.get_flag("ionod") {
        plot_ionospheric_delay(&ctx.data, plot_ctx);
    }
}
