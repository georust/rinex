mod ionex;
mod ionosphere;
mod meteo;
mod navigation;
mod observation;
mod sp3_plot;

pub use meteo::plot_meteo_observations;
pub use navigation::plot_sv_nav_clock;
pub use navigation::plot_sv_nav_orbits;
pub use observation::plot_observations;
pub use sp3_plot::plot_residual_ephemeris;

use crate::cli::Context;
use crate::graph::PlotContext;
use clap::ArgMatches;

use ionex::plot_tec_map;
use ionosphere::plot_ionospheric_delay;

pub fn plot_atmosphere_conditions(ctx: &Context, plot_ctx: &mut PlotContext, matches: &ArgMatches) {
    if matches.get_flag("tropo") {
        let _meteo = ctx.data.meteo().expect("--tropo requires METEO RINEX");
    }
    if matches.get_flag("ionod") {
        plot_ionospheric_delay(&ctx.data, plot_ctx);
    }
    if matches.get_flag("tec") {
        let ionex = ctx.data.ionex().expect("--tec required IONEX");
        plot_tec_map(ionex, ((0.0_f64, 0.0_f64), (0.0_f64, 0.0_f64)), plot_ctx);
    }
}
