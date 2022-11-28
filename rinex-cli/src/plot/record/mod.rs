use rinex::*;
use super::{
    Plot2d,
    build_plot,
    build_chart, 
    build_twoscale_chart, 
};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::{
        ChartState,
        DualCoordChartState,
    },
};
use std::collections::HashMap;

mod meteo;
mod navigation;
mod observation;

/// Plot Context for Record analysis
pub struct Context<'a> {
    /// time axis
    pub t_axis: Vec<f64>, 
    /// Plot area sorted by title
    pub plots: HashMap<String, DrawingArea<BitMapBackend<'a>, Shift>>,
    /// Single Y axes charts - indexed by titles
    pub charts: HashMap<String, ChartState<Plot2d>>,
    /// Record analysis is against time
    pub dual_charts: HashMap<String, DualCoordChartState<Plot2d, Plot2d>>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            t_axis: Vec::new(),
            charts: HashMap::new(),
            dual_charts: HashMap::new(),
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
    pub fn new (dim: (u32,u32), rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        if let Some(record) = rnx.record.as_obs() {
            observation::build_context(dim, record, nav)
        } else if let Some(record) = rnx.record.as_nav() {
            navigation::build_context(dim, record)
        } else if let Some(record) = rnx.record.as_meteo() {
            meteo::build_context(dim, record)
        } else {
            panic!("this type of record cannot be plotted yet");
        }
    }
}

/// Plots Rinex record content
pub fn plot(ctx: &mut Context, rnx: &Rinex, nav: &Option<Rinex>) {
    if let Some(record) = rnx.record.as_obs() {
        observation::plot(ctx, record, nav)
    } else if let Some(record) = rnx.record.as_nav() {
        navigation::plot(ctx, record)
    } else if let Some(record) = rnx.record.as_meteo() {
        meteo::plot(ctx, record)
    } else {
        panic!("this type of RINEX record cannot be plotted yet");
    }
}
