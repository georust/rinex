use super::{
    build_default_2y_plot, build_default_plot, build_default_polar_plot, build_world_map, Plot,
};
//use log::trace;
use plotly::{layout::MapboxStyle, Trace};

/// Plot Context
pub struct PlotContext {
    plots: Vec<Plot>,
}

impl PlotContext {
    pub fn new() -> Self {
        Self { plots: Vec::new() }
    }
    /*pub fn plot_item(&self) -> Option<&Plot> {
        self.plots.get(self.plots.len() - 1)
    }*/
    pub fn plot_item_mut(&mut self) -> Option<&mut Plot> {
        let len = self.plots.len() - 1;
        self.plots.get_mut(len)
    }
    pub fn add_cartesian2d_plot(&mut self, title: &str, y_label: &str) {
        self.plots.push(build_default_plot(title, y_label));
    }
    pub fn add_cartesian2d_2y_plot(&mut self, title: &str, y1_label: &str, y2_label: &str) {
        self.plots
            .push(build_default_2y_plot(title, y1_label, y2_label));
    }
    pub fn add_polar2d_plot(&mut self, title: &str) {
        self.plots.push(build_default_polar_plot(title));
    }
    pub fn add_world_map(&mut self, style: MapboxStyle, center: (f64, f64), zoom: u8) {
        self.plots.push(build_world_map(style, center, zoom));
    }
    pub fn add_trace(&mut self, trace: Box<dyn Trace>) {
        let len = self.plots.len() - 1;
        self.plots[len].add_trace(trace);
    }
    pub fn to_html(&mut self) -> String {
        let mut html = String::new();
        for (index, p) in self.plots.iter_mut().enumerate() {
            /*if !tiny {
                p.use_local_plotly();
            }*/
            if index == 0 {
                html.push_str(&p.to_html());
            } else {
                html.push_str(&p.to_inline_html(None));
            }
            html.push_str("\n");
        }
        html
    }
}
