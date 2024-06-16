use crate::report::shared::SamplingReport;
use itertools::Itertools;
use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskFilter, MaskOperand, Preprocessing};
use sp3::prelude::{Constellation, SP3, SV};
use std::collections::HashMap;

pub struct SP3Page {
    has_clock: bool,
    has_velocity: bool,
    has_clock_drift: bool,
    satellites: Vec<SV>,
    sampling: SamplingReport,
}

impl RenderHtml for SP3Page {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                table(class="table is-bordered") {
                    tr {
                        th(class="is-info") {
                            : "General"
                        }
                    }
                    tr {
                        th {
                            : "Velocity"
                        }
                        td {
                            : self.has_velocity.to_string()
                        }
                    }
                    tr {
                        th {
                            : "Clock offset"
                        }
                        td {
                            : self.has_clock.to_string()
                        }
                    }
                    tr {
                        th {
                            : "Clock drift"
                        }
                        td {
                            : self.has_clock_drift.to_string()
                        }
                    }
                    tr {
                        th(class="is-info") {
                            : "Satellites"
                        }
                        td {
                            : self.satellites.iter().map(|sv| sv.to_string()).join(",")
                        }
                    }
                }
            }
        }
    }
}

pub struct SP3Report {
    pub agency: String,
    pub version: String,
    pub coord_system: String,
    pub orbit_fit: String,
    pub constellation: String,
    pub time_scale: String,
    pub sampling: SamplingReport,
    pub pages: HashMap<Constellation, SP3Page>,
}

impl SP3Report {
    pub fn html_inline_menu_bar(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            a(id="menu:sp3") {
                span(class="icon") {
                    i(class="fa-solid fa-satellite");
                }
                : "High Precision Orbit (SP3)"
            }
            //ul(class="menu-list", id="menu:tabs:sp3", style="display:none") {
            //    @ for page in self.pages.keys().sorted() {
            //        li {
            //            a(id=&format!("menu:sp3:{}", page), class="tab:sp3", style="margin-left:29px") {
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
    pub fn new(sp3: &SP3) -> Self {
        Self {
            agency: sp3.agency.clone(),
            version: sp3.version.to_string(),
            coord_system: sp3.coord_system.clone(),
            orbit_fit: sp3.orbit_type.to_string(),
            time_scale: sp3.time_scale.to_string(),
            sampling: SamplingReport::from_sp3(sp3),
            constellation: sp3.constellation.to_string(),
            pages: {
                let mut pages = HashMap::<Constellation, SP3Page>::new();
                for constellation in sp3.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = sp3.filter(&filter);
                    pages.insert(
                        constellation,
                        SP3Page {
                            satellites: focused.sv().collect(),
                            has_clock: focused.sv_clock().count() > 0,
                            sampling: SamplingReport::from_sp3(&focused),
                            has_velocity: focused.sv_velocities().count() > 0,
                            has_clock_drift: focused.sv_clock_rate().count() > 0,
                        },
                    );
                }
                pages
            },
        }
    }
}

impl RenderHtml for SP3Report {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                table(class="table is-bordered") {
                    tr {
                        th {
                            : "Agency"
                        }
                        td {
                            : self.agency.clone()
                        }
                    }
                    tr {
                        th {
                            : "Constellation"
                        }
                        td {
                            : self.constellation.clone()
                        }
                    }
                    tr {
                        th {
                            : "Timescale"
                        }
                        td {
                            : self.time_scale.clone()
                        }
                    }
                    tr {
                        th {
                            : "Reference Frame"
                        }
                        td {
                            : self.coord_system.clone()
                        }
                    }
                    tr {
                        th {
                            : "Orbit FIT"
                        }
                        td {
                            : self.orbit_fit.clone()
                        }
                    }
                    tr {
                        th {
                            : "Sampling"
                        }
                        td {
                            : self.sampling.to_inline_html()
                        }
                    }
                }//table
            }//table-container
            @ for (constell, page) in self.pages.iter() {
                div(class="table-container is-page", id=&format!("sp3:{}", constell), style="display:none") {
                    table(class="table is-bordered") {
                        tr {
                            th(class="is-info") {
                                : constell.to_string()
                            }
                        }
                        tr {
                            : page.to_inline_html()
                        }
                    }
                }
            }
        }
    }
}
