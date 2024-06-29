use crate::plot::Plot;
use crate::prelude::QcContext;
use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::prelude::{Constellation, Epoch, Rinex};
use std::collections::HashMap;

struct SignalPlot {
    pub spp_plot: Plot,
    pub cpp_plot: Plot,
    pub ppp_plot: Plot,
}

impl Render for SignalPlot {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        td {
                            (self.spp_plot.render())
                        }
                    }
                }
            }
        }
    }
}

impl SignalPlot {
    pub fn new(obs_rinex: &Rinex, context: &QcContext) -> Self {
        let mut spp_plot = Plot::new_time_domain("test", "test", "test", true);
        let mut spp_ok_t = Vec::<Epoch>::new();
        let mut spp_ok_y = Vec::<u8>::new();
        let mut spp_no_nav_t = Vec::<Epoch>::new();
        let mut spp_no_nav_y = Vec::<u8>::new();

        let mut cpp_plot = Plot::new_time_domain("test", "test", "test", true);
        let mut ppp_plot = Plot::new_time_domain("test", "test", "test", true);
        for prn in obs_rinex.sv().map(|sv| sv.prn) {}
        Self {
            spp_plot,
            cpp_plot,
            ppp_plot,
        }
    }
}

pub struct CombinationPlot {
    pub spp_plot: Plot,
    pub cpp_plot: Plot,
    pub ppp_plot: Plot,
}

impl Render for CombinationPlot {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        td {
                            (self.spp_plot.render())
                        }
                    }
                }
            }
        }
    }
}

impl CombinationPlot {
    pub fn new(obs_rinex: &Rinex, context: &QcContext) -> Self {
        let mut spp_plot = Plot::new_time_domain("test", "test", "test", true);
        let mut cpp_plot = Plot::new_time_domain("test", "test", "test", true);
        let mut ppp_plot = Plot::new_time_domain("test", "test", "test", true);
        let mut spp_ok_t = Vec::<Epoch>::new();
        let mut spp_ok_y = Vec::<u8>::new();
        let mut spp_no_nav_t = Vec::<Epoch>::new();
        Self {
            spp_plot,
            cpp_plot,
            ppp_plot,
        }
    }
}

impl ConstellationPlots {
    pub fn new(obs_rinex: &Rinex, context: &QcContext) -> Self {
        let mut signals = HashMap::<String, SignalPlot>::new();
        let mut combinations = HashMap::<String, CombinationPlot>::new();
        for signal in obs_rinex.carrier() {}
        Self {
            signals,
            combinations,
        }
    }
}

struct ConstellationPlots {
    /// One plot per signal
    pub signals: HashMap<String, SignalPlot>,
    /// One plot per combination
    pub combinations: HashMap<String, CombinationPlot>,
}

impl Render for ConstellationPlots {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    @for signal in self.signals.keys().sorted() {
                        @if let Some(page) = self.signals.get(signal) {
                            tr {
                                th class="is-info" {
                                    (signal.to_string())
                                }
                                td {
                                    (page.render())
                                }
                            }
                        }
                    }
                    @for signal in self.combinations.keys().sorted() {
                        @if let Some(page) = self.signals.get(signal) {
                            tr {
                                th class="is-info" {
                                    (signal.to_string())
                                }
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
}

pub struct QcNavi {
    pages: HashMap<Constellation, ConstellationPlots>,
}

impl QcNavi {
    pub fn new(context: &QcContext) -> Self {
        let mut pages = HashMap::<Constellation, ConstellationPlots>::new();
        let observation = context.observation().unwrap();
        for constell in observation.constellation() {
            let filter = Filter::mask(
                MaskOperand::Equals,
                FilterItem::ConstellationItem(vec![constell]),
            );
            let focused = observation.filter(&filter);
            pages.insert(constell, ConstellationPlots::new(&focused, context));
        }
        Self { pages }
    }
}

impl Render for QcNavi {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    @for constell in self.pages.keys().sorted() {
                        @if let Some(page) = self.pages.get(&constell) {
                            tr {
                                th class="is-info" {
                                    (constell.to_string())
                                }
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
}
