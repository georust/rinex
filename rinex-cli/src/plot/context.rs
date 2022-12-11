use super::{
    Plot,
    build_default_plot,
};
use std::io::Write;
use plotly::{
    Trace,
    layout::{
        Layout, LayoutGrid, GridPattern,
    },
    common::Title,
};

pub enum PlotType {
    Cartesian2d,
    Polar2d,
}

/// CLI Plot Context 
pub struct Context {
    plots: Vec<Plot>, 
}

impl Context {
    pub fn new() -> Self {
        Self {
            plots: Vec::new(),
        }
    }
    pub fn add_plot(&mut self, plot_type: PlotType, title: &str, y_label: &str) {
        match plot_type {
            PlotType::Cartesian2d => self.plots.push(build_default_plot(title, y_label)),
            PlotType::Polar2d => {}, //self.plots.push(build_polar2d(title, y_label)),
        }
    }
    pub fn add_trace(&mut self, trace: Box<dyn Trace>) {
        let len = self.plots.len()-1;
        self.plots[len].add_trace(trace);
    }
    pub fn to_html(&self) {
        let mut html = String::new();
        for (index, p) in self.plots.iter().enumerate() {
            if index == 0 {
                html.push_str(&p.to_html());
            } else {
                html.push_str(&p.to_inline_html(None));
            }
            html.push_str("\n");
        }
        let mut fd = std::fs::File::create("/tmp/test.html")
            .unwrap();
        write!(fd, "{}", html)
            .unwrap();
    }
    pub fn show(&self) {
    }
}
