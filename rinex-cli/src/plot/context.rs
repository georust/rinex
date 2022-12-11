use super::{build_default_plot, build_default_polar_plot, Plot};
use plotly::Trace;

/// CLI Plot Context
pub struct Context {
    plots: Vec<Plot>,
}

impl Context {
    pub fn new() -> Self {
        Self { plots: Vec::new() }
    }
    pub fn add_cartesian2d_plot(&mut self, title: &str, y_label: &str) {
        self.plots.push(build_default_plot(title, y_label));
    }
    pub fn add_polar2d_plot(&mut self, title: &str) {
        self.plots.push(build_default_polar_plot(title));
    }
    pub fn add_trace(&mut self, trace: Box<dyn Trace>) {
        let len = self.plots.len() - 1;
        self.plots[len].add_trace(trace);
    }
    pub fn to_html(&mut self, tiny: bool) -> String {
        let mut html = String::new();
        for (index, p) in self.plots.iter_mut().enumerate() {
            if !tiny {
                p.use_local_plotly();
            }
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
