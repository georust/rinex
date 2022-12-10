use rinex::prelude::*;
use plotly::{
    Plot, Histogram,
};

/*
 * Epoch duration histogram
 */
pub fn epoch_histogram(rnx: &Rinex) {
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
    let mut plot = Plot::new();
    plot.add_trace(histogram);
    plot.show();
    /*chart
        .draw_series(
            Histogram::vertical(&chart).data(
                histogram
                    .iter()
                    .map(|(duration, pop)| (duration.to_seconds() as u32, *pop)),
            ),
        )
        .expect("failed to draw histogram");*/
}
