use rinex_qc::prelude::{html, Markup, Plot, QcContext, Render};
use rtk::prelude::{Carrier, Config, Epoch, Method, PVTSolution, SV};
use std::collections::{BTreeMap, HashMap};

/// Solutions report
pub struct Report {
    refsys_plot: Plot,
    sv_plot: Plot,
    ionod_plot: Plot,
    tropod_plot: Plot,
}

impl Report {
    pub fn new(ctx: &QcContext, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            refsys_plot: Plot::new_time_domain("test", "test", "test", true),
            sv_plot: Plot::new_time_domain("test", "test", "test", true),
            ionod_plot: Plot::new_time_domain("test", "test", "test", true),
            tropod_plot: Plot::new_time_domain("test", "test", "test", true),
        }
    }
}

impl Render for Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
            }
        }
    }
}
