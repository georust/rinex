use super::{build_chart, build_plot, Plot2d};
use plotters::{chart::ChartState, coord::Shift, prelude::*};
use rinex::*;
use std::collections::HashMap;

pub mod meteo;
pub mod ionex;
pub mod navigation;
pub mod observation;

/// Plot Context for Record analysis
pub struct Context<'a> {
    /// Plot area sorted by title
    pub plots: HashMap<String, DrawingArea<BitMapBackend<'a>, Shift>>,
    /// Charts are indexed by sub titles
    pub charts: HashMap<String, ChartState<Plot2d>>,
    /// Record analysis is against time
    pub t_axis: Vec<f64>,
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self {
            t_axis: Vec::new(),
            charts: HashMap::new(),
            plots: HashMap::new(),
        }
    }
}
