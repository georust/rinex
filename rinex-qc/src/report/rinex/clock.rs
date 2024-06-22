use crate::report::shared::SamplingReport;
use crate::report::Error;
// use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::prelude::{ClockProfileType, Constellation, Rinex, TimeScale, DOMES, SV};
use std::collections::HashMap;

pub struct ClkPage {
    satellites: Vec<SV>,
}

pub struct ClkReport {
    site: Option<String>,
    domes: Option<DOMES>,
    clk_name: Option<String>,
    sampling: SamplingReport,
    ref_clock: Option<String>,
    codes: Vec<ClockProfileType>,
    igs_clock_name: Option<String>,
    timescale: Option<TimeScale>,
    pages: HashMap<Constellation, ClkPage>,
}

impl ClkReport {
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:clk" {
                span class="icon" {
                    i class="fa-solid fa-clock" {}
                }
                "High Precision Clock (RINEX)"
            }
            //ul(class="menu-list", id="menu:tabs:clk", style="display:none") {
            //    @ for page in self.pages.keys().sorted() {
            //        li {
            //            a(id=&format!("menu:clk:{}", page), style="margin-left:29px") {
            //                span(class="icon") {
            //                    i(class="fa-solid fa-satellite");
            //                }
            //                : page.to_string()
            //            }
            //        }
            //    }
            //}
        }
    }
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let clk_header = rnx.header.clock.as_ref().ok_or(Error::MissingClockHeader)?;
        Ok(Self {
            site: clk_header.site.clone(),
            domes: clk_header.domes.clone(),
            codes: clk_header.codes.clone(),
            igs_clock_name: clk_header.igs.clone(),
            clk_name: clk_header.full_name.clone(),
            ref_clock: clk_header.ref_clock.clone(),
            timescale: clk_header.timescale.clone(),
            sampling: SamplingReport::from_rinex(rnx),
            pages: {
                let mut pages = HashMap::<Constellation, ClkPage>::new();
                for constellation in rnx.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = rnx.filter(&filter);
                    pages.insert(
                        constellation,
                        ClkPage {
                            satellites: focused.sv().collect(),
                        },
                    );
                }
                pages
            },
        })
    }
}

impl Render for ClkReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        @if let Some(clk_name) = &self.clk_name {
                            tr {
                                th class="is-info" {
                                    "Agency"
                                }
                                td {
                                    (clk_name)
                                }
                            }
                        }
                        @if let Some(site) = &self.site {
                            tr {
                                th {
                                    "Clock Site"
                                }
                                td {
                                    (site)
                                }
                            }
                        }
                        @if let Some(domes) = &self.domes {
                            tr {
                                th {
                                    "DOMES #ID"
                                }
                                td {
                                    (domes.to_string())
                                }
                            }
                        }
                        @if let Some(ref_clk) = &self.ref_clock {
                            tr {
                                th {
                                    "Reference Clock"
                                }
                                td {
                                    (ref_clk)
                                }
                            }
                        }
                        @if let Some(igs_name) = &self.igs_clock_name {
                            tr {
                                th {
                                    "IGS Clock Name"
                                }
                                td {
                                    (igs_name)
                                }
                            }
                        }
                        @if let Some(timescale) = self.timescale {
                            tr {
                                th {
                                    "Timescale"
                                }
                                td {
                                    (timescale.to_string())
                                }
                            }
                        }
                    }
                }
            }
            div class="table-container" {
                (self.sampling.render())
            }
        }
    }
}
