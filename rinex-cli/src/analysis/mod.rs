pub mod sv_epoch;
//use crate::plot::*;
//use plotters::prelude::*;
use rinex::prelude::*;


pub fn epoch_histogram(rnx: &Rinex, dims: (u32, u32)) {
/*    let histogram = rnx.epoch_intervals();
    let p = build_plot("epoch-histogram.png", dims);
    let mut pop_max: u32 = 0;
    let mut duration_max = 0_u32;
    for (duration, pop) in &histogram {
        if *pop > pop_max {
            pop_max = *pop;
        }
        let seconds = duration.to_seconds() as u32;
        if seconds > duration_max {
            duration_max = seconds;
        }
    }
    let mut chart = ChartBuilder::on(&p)
        .caption("Epoch Durations", ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(0..duration_max, 0..pop_max)
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
    chart
        .draw_series(
            Histogram::vertical(&chart).data(
                histogram
                    .iter()
                    .map(|(duration, pop)| (duration.to_seconds() as u32, *pop)),
            ),
        )
        .expect("failed to draw histogram");*/
}
