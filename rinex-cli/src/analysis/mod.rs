pub mod sv_epoch;
use crate::plot::*;
use rinex::prelude::*;
use plotters::prelude::*;

use chrono::{
    Date,
    Duration,
};

/*
 * Converts an Hifitime::Epoch to a plottable Chrono::Date
 */
macro_rules! to_date {
    ($epoch: expr) => {
        
    }
}

/*
 * Converts an Hifitime::Duration to a plottable Chrono::Duration
 */
macro_rules! to_duration {
    ($duration: expr) => {

    }
}

pub fn epoch_histogram(rnx: &Rinex, dims: (u32, u32)) {
    let histogram = rnx.epoch_intervals();
    let p = build_plot("epoch-histogram.png", dims); 
    let mut pop_max: u32 = 0;
    let mut duration_max: u32 = 0;
    for (duration, pop) in histogram {
        if pop > pop_max {
            pop_max = pop;
        }
        if duration > duration_max {
            duration_max = duration;
        }
    }
    /*
    let mut chart = ChartBuilder::on(&p)
        .caption("Epoch Durations", ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_axis, y_axis)
        .unwrap();
    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .y_desc("Count (population)")
        .x_desc("Duration [s]")
        .axis_desc_style(("sans-serif", 15))
        .draw()
        .expect("failed to draw mesh");
    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(histogram.iter().map(|pop, duration| (*duration, 1)));*/
}
