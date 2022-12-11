use rinex::prelude::*;
use crate::plot::{
    Context,
};
use plotly::{
    Plot, Histogram,
};

/*
 * Epoch duration histogram
 */
pub fn epoch_histogram(ctx: &mut Context, rnx: &Rinex) {
    // create a new plot
    ctx.add_cartesian2d_plot("Epoch Intervals", "Count");
    let histogram = rnx.epoch_intervals();
    let mut durations: Vec<&Duration> = histogram
        .keys()
        .collect();
    durations.sort();
    let durations: Vec<String> = durations
        .iter()
        .map(|k| k.to_string())
        .collect();
    let pop: Vec<_> = histogram
        .values()
        .map(|v| v.to_string())
        .collect();
    let histogram = Histogram::new_xy(durations, pop)
        .name("Epoch Intervals");
    ctx.add_trace(histogram);
}
