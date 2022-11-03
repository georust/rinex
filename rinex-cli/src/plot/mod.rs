pub mod record;
pub mod differential;

use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
    coord::types::RangedCoordf64,
};
use std::collections::HashMap;

pub type Plot2d = Cartesian2d<RangedCoordf64, RangedCoordf64>;
    
/// Builds plot area
pub fn build_plot(file: &str, dims: (u32,u32)) -> DrawingArea<BitMapBackend, Shift> {
    let area = BitMapBackend::new(file, dims)
        .into_drawing_area();
    area.fill(&WHITE)
        .expect("failed to create background image");
    area
}

/// Builds a chart
pub fn build_chart(title: &str, x_axis: Vec<f64>, y_range: (f64,f64), 
        area: &DrawingArea<BitMapBackend, Shift>) 
            -> ChartState<Plot2d> 
{
    let x_axis = x_axis[0]..x_axis[x_axis.len()-1]; 
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_axis, 0.98*y_range.0..1.02*y_range.1) // nicer Y scale
        .unwrap();
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]") //TODO not for special records
        .x_labels(30)
        .y_desc(title)
        .y_labels(30)
        .draw()
        .unwrap();
    chart
        .to_chart_state()
}

