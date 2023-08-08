use crate::{plot::PlotContext, Context};
use itertools::Itertools;
use plotly::Histogram; //.sorted()

/*
 * Sampling histogram
 */
pub fn histogram(ctx: &Context, plot_ctx: &mut PlotContext) {
    plot_ctx.add_cartesian2d_plot("Sampling Histogram", "Count");
    let durations: Vec<_> = ctx
        .primary_rinex
        .sampling_histogram()
        .sorted()
        .map(|(dt, _)| dt.to_string())
        .collect();
    let populations: Vec<_> = ctx
        .primary_rinex
        .sampling_histogram()
        .sorted()
        .map(|(_, pop)| pop.to_string())
        .collect();
    let histogram = Histogram::new_xy(durations, populations).name("Sampling Histogram");
    plot_ctx.add_trace(histogram);

    if let Some(ref nav) = ctx.nav_rinex {
        // Run similar analysis on NAV context
        let durations: Vec<_> = nav
            .sampling_histogram()
            .sorted()
            .map(|(dt, _)| dt.to_string())
            .collect();
        let populations: Vec<_> = nav
            .sampling_histogram()
            .sorted()
            .map(|(_, pop)| pop.to_string())
            .collect();
        let histogram = Histogram::new_xy(durations, populations).name("(NAV) Sampling Histogram");
        plot_ctx.add_trace(histogram);
    }
}
