use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
    coord::types::RangedCoordf64,
};
use std::collections::HashMap;

use rinex::*;
mod meteo;
mod navigation;
mod observation;

pub type Plot2d = Cartesian2d<RangedCoordf64, RangedCoordf64>;
 
pub struct Context<'a> {
    /// Plots are "Drawing areas" that we can either
    /// draw basic structures on, or stack Charts
    /// and 3D widgets onto.
    /// Plots are sorted by their titles which should always
    /// be a meaningful value
    pub plots: HashMap<String, DrawingArea<BitMapBackend<'a>, Shift>>,
    /// Drawing charts,
    /// is where actual plotting happens.
    /// We only work with f64 data
    pub charts: HashMap<String, ChartState<Plot2d>>,
    /// All plots share same time axis
    pub t_axis: Vec<f64>, 
    /// Colors used when plotting
    pub colors: HashMap<String, RGBAColor>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            t_axis: Vec::new(),
            colors: HashMap::new(),
            charts: HashMap::new(),
            plots: HashMap::new(),
        }
    }
}

impl<'a> Context<'a> {
    /// Builds a new plotting context
    ///  Iterates the RINEX context once (for overall performance considerations).
    ///  Prepares Plot and Charts depending on given RINEX context.
    ///  Currently all `Epoch` sorted RINEX have a time axis
    ///  in second, that eventually should be improved to exhibit
    ///  the real Date object. It seems possible to plot a Date<Local>
    ///  with the libs we're using.
    ///
    ///  Dim: (u32, u32) plot x_width and y_height
    pub fn new (dim: (u32,u32), rnx: &Rinex) -> Self {
        if let Some(record) = rnx.record.as_obs() {
            observation::build_context(dim, record)
        } else if let Some(record) = rnx.record.as_nav() {
            navigation::build_context(dim, record)
        } else if let Some(record) = rnx.record.as_meteo() {
            meteo::build_context(dim, record)
        } else {
            panic!("this type of record cannot be plotted yet");
        }
    }
    
    /// Build plot
    pub fn build_plot(file: &str, dim: (u32,u32)) -> DrawingArea<BitMapBackend, Shift> {
        let area = BitMapBackend::new(file, dim)
            .into_drawing_area();
        area.fill(&WHITE)
            .expect("failed to create background image");
        area
    }
    
    /// Build Charts
    pub fn build_chart(title: &str, 
        x_axis: Vec<f64>, 
        y_range: (f64,f64), 
        area: &DrawingArea<BitMapBackend, Shift>) 
    -> ChartState<Plot2d> {
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
}

pub fn plot_rinex(ctx: &mut Context, rnx: &Rinex) {
    if let Some(record) = rnx.record.as_obs() {
        observation::plot(ctx, record)
    } else if let Some(record) = rnx.record.as_nav() {
        navigation::plot(ctx, record)
    } else if let Some(record) = rnx.record.as_meteo() {
        meteo::plot(ctx, record)
    } else {
        panic!("this type of RINEX record cannot be plotted yet");
    }
}
