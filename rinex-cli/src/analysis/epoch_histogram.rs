use crate::{plot::PlotContext, Context};
use plotly::Histogram;
use rinex::prelude::*;

/*
 * Epoch duration histogram
 */
pub fn epoch_histogram(ctx: &Context, plot_ctx: &mut PlotContext) {
    plot_ctx.add_cartesian2d_plot("Epoch Intervals", "Count");
    let histogram = ctx.primary_rinex.epoch_intervals();
    let mut durations: Vec<&Duration> = histogram.keys().collect();
    durations.sort();

    let durations: Vec<String> = durations.iter().map(|k| k.to_string()).collect();
    let pop: Vec<_> = histogram.values().map(|v| v.to_string()).collect();
    let histogram = Histogram::new_xy(durations, pop).name("Epoch Intervals");
    plot_ctx.add_trace(histogram);

    if let Some(ref nav) = ctx.nav_rinex {
        let histogram = nav.epoch_intervals();
        let mut durations: Vec<&Duration> = histogram.keys().collect();
        durations.sort();

        let durations: Vec<String> = durations.iter().map(|k| k.to_string()).collect();
        let pop: Vec<_> = histogram.values().map(|v| v.to_string()).collect();
        let histogram = Histogram::new_xy(durations, pop).name("(NAV) Epoch Intervals");
        plot_ctx.add_trace(histogram);
    }
}
