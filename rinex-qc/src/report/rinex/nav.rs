use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::prelude::{Constellation, Rinex, SV};
use std::collections::HashMap;

#[cfg(feature = "sp3")]
use crate::plot::Plot;

struct ConstellationPage {
    satellites: Vec<SV>,
    #[cfg(feature = "plot")]
    sky_plot: Option<Plot>,
}

impl ConstellationPage {
    fn new(rinex: &Rinex, sky_plot: bool) -> Self {
        Self {
            satellites: rinex.sv().collect(),
            #[cfg(feature = "plot")]
            sky_plot: {
                if sky_plot {
                    let mut plot = Plot::sky_plot("brdc_sky", "Skyplot", true);
                    Some(plot)
                } else {
                    None
                }
            },
        }
    }
}

impl Render for ConstellationPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Satellites"
                        }
                        td {
                            (self.satellites.iter().join(", "))
                        }
                    }
                    @if let Some(plot) = &self.sky_plot {
                        tr {
                            th class="is-info" {
                                "Sky Compass"
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
}

pub struct NavReport {
    pages: HashMap<Constellation, ConstellationPage>,
}

impl NavReport {
    pub fn new(rinex: &Rinex, sky_plot: bool) -> Self {
        Self {
            pages: {
                let mut pages = HashMap::<Constellation, ConstellationPage>::new();
                for constell in rinex.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constell]),
                    );
                    let focused = rinex.filter(&filter);
                    pages.insert(constell, ConstellationPage::new(&focused, sky_plot));
                }
                pages
            },
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:brdc" {
                span class="icon" {
                    i class="fa-solid fa-satellite-dish" {}
                }
                "Broadcast Navigation (BRDC)"
            }
            ul class="menu-list" id="menu:tabs:brdc" style="display:none" {
                @for page in self.pages.keys().sorted() {
                    li {
                        a id=(format!("menu:brdc:{}", page)) style="margin-left:29px" {
                            span class="icon" {
                                i class="fa-solid fa-satellite" {}
                            }
                            (page.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl Render for NavReport {
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
