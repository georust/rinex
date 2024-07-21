use crate::cli::Context;
use rinex_qc::prelude::{html, Markup, Plot, Render};

use cggtts::prelude::Track;

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

/// Solutions report
struct ReportContent {
    refsys_plot: Plot,
    sv_plot: Plot,
    ionod_plot: Plot,
    tropod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        Self {
            refsys_plot: Plot::timedomain_plot("test", "test", "test", true),
            sv_plot: Plot::timedomain_plot("test", "test", "test", true),
            ionod_plot: Plot::timedomain_plot("test", "test", "test", true),
            tropod_plot: Plot::timedomain_plot("test", "test", "test", true),
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

pub struct Report {
    tab: ReportTab,
    content: ReportContent,
}

impl Report {
    //pub fn formalize(&self) -> QcExtraPage {
    //    QcExtraPage {
    //        tab: Box::new(self.tab.clone()),
    //        content: Box::new(self.content.clone()),
    //    }
    //}
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
