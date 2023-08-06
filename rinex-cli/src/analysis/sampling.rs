use crate::{plot::PlotContext, Context};
use plotly::Histogram;
use rinex::prelude::*;

/*
 * Sampling histogram
 */
pub fn histogram(ctx: &Context, plot_ctx: &mut PlotContext) {
    plot_ctx.add_cartesian2d_plot("Sampling Histogram", "Count");
    let histogram = ctx.primary_rinex.sampling_histogram();
    let mut durations: Vec<&Duration> = histogram.keys().collect();
    durations.sort();

    let pop: Vec<_> = histogram.values().map(|v| v.to_string()).collect();
    let durations: Vec<String> = durations.iter().map(|k| k.to_string()).collect();
    let histogram = Histogram::new_xy(durations, pop).name("Sampling Histogram");
    plot_ctx.add_trace(histogram);

    if let Some(ref nav) = ctx.nav_rinex {
        let histogram = nav.sampling_histogram();
        let mut durations: Vec<&Duration> = histogram.keys().collect();
        durations.sort();

        let pop: Vec<_> = histogram.values().map(|v| v.to_string()).collect();
        let durations: Vec<String> = durations.iter().map(|k| k.to_string()).collect();
        let histogram = Histogram::new_xy(durations, pop).name("(NAV) Sampling Histogram");
        plot_ctx.add_trace(histogram);
    }
}
