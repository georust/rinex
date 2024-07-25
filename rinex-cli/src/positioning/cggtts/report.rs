use crate::cli::Context;
use rinex_qc::prelude::{html, Marker, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render};

use cggtts::prelude::Track;

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

/// Solutions report
struct ReportContent {
    sv_plot: Plot,
    ionod_plot: Plot,
    refsys_plot: Plot,
    tropod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        Self {
            sv_plot: Plot::timedomain_plot("test", "test", "test", true),
            ionod_plot: Plot::timedomain_plot("test", "test", "test", true),
            tropod_plot: Plot::timedomain_plot("test", "test", "test", true),
            refsys_plot: Plot::timedomain_plot("test", "test", "test", true),
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
    pub fn formalize(self) -> QcExtraPage {
        QcExtraPage {
            tab: Box::new(self.tab),
            content: Box::new(self.content),
        }
    }
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
