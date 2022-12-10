use super::{
    Plot,
    build_default_plot,
};
use plotly::{
    Trace,
    layout::{
        Layout, LayoutGrid, GridPattern,
    },
    common::Title,
};

pub struct Context {
    nb_plots: usize,
    plot: Plot, 
}

impl Context {
    pub fn new() -> Self {
        Self {
            nb_plots: 0,
            plot: Plot::new(),
        }
    }
    pub fn add_plot(&mut self, title: &str, y_label: &str) {
        self.nb_plots += 1;
        if self.nb_plots == 1 {
            self.plot = build_default_plot(title, y_label);
        }
        /*} else {
            let layout = self.plot.layout();
            let new_layout = layout.clone()
                .title(Title::new("test"));
            self.plot.set_layout(new_layout);
        }*/
    }
    pub fn add_trace(&mut self, trace: Box<dyn Trace>) {
        self.plot.add_trace(trace);
    }
    pub fn show(&mut self) {
        let layout = self.plot.layout();
        let new = layout.clone().grid(
            LayoutGrid::new()
                .rows(self.nb_plots)
                .columns(1)
                .y_gap(200.0)
                .pattern(GridPattern::Independent),
        );
        self.plot.set_layout(new);
        self.plot.show();
    }
}
