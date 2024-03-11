use crate::graph::PlotContext;
use itertools::Itertools;
use plotly::Histogram;
use rinex::prelude::{ProductType, RnxContext};

/*
 * Sampling histogram
 */
pub fn histogram(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    plot_ctx.add_timedomain_plot("Sampling Histogram", "Count");
    for product in [
        ProductType::Observation,
        ProductType::MeteoObservation,
        ProductType::BroadcastNavigation,
        ProductType::HighPrecisionClock,
        ProductType::Ionex,
    ] {
        if let Some(data) = ctx.rinex(product) {
            let histogram = data.sampling_histogram().sorted();
            let durations: Vec<_> = histogram.clone().map(|(dt, _)| dt.to_string()).collect();
            let populations: Vec<_> = histogram.clone().map(|(_, pop)| pop.to_string()).collect();
            let histogram = Histogram::new_xy(durations, populations).name(&format!("{}", product));
            plot_ctx.add_trace(histogram);
        }
    }
}
