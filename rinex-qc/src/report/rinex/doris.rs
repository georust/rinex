use crate::report::shared::SamplingReport;
use itertools::Itertools;
use maud::{html, Markup, Render};
use std::collections::HashMap;

#[cfg(feature = "plot")]
use crate::plot::Plot;

use rinex::{
    carrier::Carrier,
    prelude::{Constellation, Observable, Rinex, SV},
};

struct SignalPage {
    /// Sampling
    sampling: SamplingReport,
    /// one plot per physics
    #[cfg(feature = "plot")]
    raw_plots: HashMap<Observable, Plot>,
}

impl Render for SignalPage {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tr {
                    th class="is-info" {
                        "Sampling"
                    }
                    td {
                        (self.sampling.render())
                    }
                }
                @for observable in self.raw_plots.keys().sorted() {
                    @if let Some(plot) = self.raw_plots.get(observable) {
                        th class="is-info" {
                            (format!("{} Observations", observable))
                        }
                        td {
                            (plot.render())
                        }
                    }
                }
            }
        }
    }
}

/// Doris RINEX QC
pub struct DorisReport {
    sampling: SamplingReport,
    signals: HashMap<Carrier, SignalPage>,
}

impl DorisReport {
    pub fn new(rinex: &Rinex) -> Self {
        Self {
            sampling: SamplingReport::from_rinex(rinex),
            signals: {
                let mut signals = HashMap::<Carrier, SignalPage>::new();
                signals
            },
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:doris" {
                span class="icon" {
                    i class="fa-solid fa-tower-cell" {}
                }
                "DORIS Observations"
            }
        }
    }
}

impl Render for DorisReport {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tr {
                    th class="is-info" {
                        "Sampling"
                    }
                    td {
                        (self.sampling.render())
                    }
                }
                @for signal in self.signals.keys().sorted() {
                    @if let Some(page) = self.signals.get(signal) {
                        tr {
                            td {
                                (page.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
