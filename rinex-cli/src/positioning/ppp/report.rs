use crate::cli::Context;
use rinex_qc::prelude::{html, Markup, Plot, QcContext, QcExtraPage, Render};
use rtk::prelude::{Carrier, Config, Epoch, Method, PVTSolution, SV};
use std::collections::{BTreeMap, HashMap};

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

struct ReportContent {
    refsys_plot: Plot,
    sv_plot: Plot,
    ionod_plot: Plot,
    tropod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            refsys_plot: Plot::new_time_domain("test", "test", "test", true),
            sv_plot: Plot::new_time_domain("test", "test", "test", true),
            ionod_plot: Plot::new_time_domain("test", "test", "test", true),
            tropod_plot: Plot::new_time_domain("test", "test", "test", true),
        }
    }
}

impl Render for ReportContent {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
            }
        }
    }
}

/// Solutions report
pub struct Report {
    tab: ReportTab,
    content: ReportContent,
}

impl Report {
    pub fn formalize(&self) -> QcExtraPage {
        QcExtraPage {
            tab: Box::new(self.tab),
            content: Box::new(self.content),
        }
    }
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
