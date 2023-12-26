use crate::plot::PlotContext;
use itertools::Itertools;
use plotly::Histogram; //.sorted()
use rinex::prelude::RnxContext;

/*
 * Sampling histogram
 */
pub fn histogram(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    plot_ctx.add_timedomain_plot("Sampling Histogram", "Count");
    if let Some(data) = ctx.obs_data() {
        let histogram = data.sampling_histogram().sorted();
        let durations: Vec<_> = histogram.clone().map(|(dt, _)| dt.to_string()).collect();
        let populations: Vec<_> = histogram.clone().map(|(_, pop)| pop.to_string()).collect();
        let histogram = Histogram::new_xy(durations, populations).name("Sampling Histogram");
        plot_ctx.add_trace(histogram);
    }
    // Run similar analysis on NAV context
    if let Some(data) = &ctx.nav_data() {
        let histogram = data.sampling_histogram().sorted();
        let durations: Vec<_> = histogram.clone().map(|(dt, _)| dt.to_string()).collect();
        let populations: Vec<_> = histogram.clone().map(|(_, pop)| pop.to_string()).collect();
        let histogram = Histogram::new_xy(durations, populations).name("(NAV) Sampling Histogram");
        plot_ctx.add_trace(histogram);
    }
}
