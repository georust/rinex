use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};

use itertools::Itertools;
use std::collections::HashMap;

use rinex::{
    carrier::Carrier,
    hardware::{Antenna, Receiver},
    prelude::{Constellation, Duration, Epoch, Observable, Rinex},
};

use crate::plot::Plot;
use crate::report::shared::SamplingReport;

/// Frequency dependent pagination
pub struct FrequencyPage {
    /// Sampling
    sampling: SamplingReport,
}

impl RenderHtml for FrequencyPage {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table is-bordered") {
                tr {
                    th(class="is-info") {
                        : "Sampling"
                    }
                    td {
                        : self.sampling.to_inline_html()
                    }
                }
            }
        }
    }
}

/// Constellation dependent pagination
pub struct ConstellationPage {
    /// True when doppler are sampled
    pub doppler: bool,
    /// True if Standard Positioning compatible
    pub spp_compatible: bool,
    /// True if Code Dual Frequency Positioning compatible
    pub cpp_compatible: bool,
    /// True if PPP compatible
    pub ppp_compatible: bool,
    /// Frequency dependent pagination
    pub pages: HashMap<Carrier, FrequencyPage>,
}

impl RenderHtml for ConstellationPage {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                table(class="table is-bordered") {
                    tbody {
                        tr {
                            th {
                                : "Has Doppler"
                            }
                            td {
                                : self.doppler.to_string()
                            }
                            @ if self.doppler {
                                td {
                                    span(class="icon", style="color:green") {
                                        i(class="fa-solid fa-circle-check");
                                    }
                                }
                            } else {
                                td {
                                    span(class="icon", style="color:red") {
                                        i(class="fa-solid fa-circle-xmark");
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                : "SPP Compatible"
                            }
                            @ if self.spp_compatible {
                                td {
                                    span(class="icon", style="color:green") {
                                        i(class="fa-solid fa-circle-check");
                                    }
                                }
                            } else {
                                td {
                                    span(class="icon", style="color:red") {
                                        i(class="fa-solid fa-circle-xmark");
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                : "CPP compatible"
                            }
                            @ if self.cpp_compatible {
                                td {
                                    span(class="icon", style="color:green") {
                                        i(class="fa-solid fa-circle-check");
                                    }
                                }
                            } else {
                                td {
                                    span(class="icon", style="color:red") {
                                        i(class="fa-solid fa-circle-xmark");
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                : "PPP compatible"
                            }
                            @ if self.ppp_compatible {
                                span(class="icon", style="color:green") {
                                    i(class="fa-solid fa-circle-check");
                                }
                            } else {
                                span(class="icon", style="color:red") {
                                    i(class="fa-solid fa-circle-xmark");
                                }
                            }
                        }
                        @ for carrier in self.pages.keys().sorted() {
                            @ if let Some(page) = self.pages.get(carrier) {
                                tr {
                                    th(class="is-info") {
                                        : carrier.to_string()
                                    }
                                    td {
                                        : page.to_inline_html()
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// RINEX Observation Report
pub struct Report {
    antenna: Option<Antenna>,
    receiver: Option<Receiver>,
    sampling: SamplingReport,
    pages: HashMap<Constellation, ConstellationPage>,
}

impl Report {
    pub fn html_inline_menu_bar(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            a(id="menu:obs") {
                span(class="icon") {
                    i(class="fa-solid fa-tower-cell");
                }
                : "Observations"
            }
            //ul(class="menu-list", id="menu:tabs:obs", style="display:none") {
            //    @ for constell in self.pages.keys() {
            //        li {
            //            a(id=&format!("menu:obs:{}", constell), style="margin-left:29px") {
            //                span(class="icon") {
            //                    i(class="fa-solid fa-satellite");
            //                }
            //                : constell.to_string()
            //            }
            //        }
            //    }
            //}
        }
    }
    pub fn new(rinex: &Rinex) -> Self {
        Self {
            sampling: SamplingReport::from_rinex(rinex),
            receiver: if let Some(rcvr) = &rinex.header.rcvr {
                Some(rcvr.clone())
            } else {
                None
            },
            antenna: if let Some(ant) = &rinex.header.rcvr_antenna {
                Some(ant.clone())
            } else {
                None
            },
            pages: {
                let mut pages = HashMap::<Constellation, ConstellationPage>::new();
                for constellation in rinex.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = rinex.filter(&filter);
                    pages.insert(
                        constellation,
                        ConstellationPage {
                            doppler: focused.doppler().count() > 0,
                            spp_compatible: false,
                            cpp_compatible: false,
                            ppp_compatible: false,
                            pages: {
                                let mut pages = HashMap::<Carrier, FrequencyPage>::new();
                                for observable in focused.observable() {
                                    if let Ok(carrier) =
                                        Carrier::from_observable(constellation, observable)
                                    {
                                        let filter = Filter::mask(
                                            MaskOperand::Equals,
                                            FilterItem::ComplexItem(vec![observable.to_string()]),
                                        );
                                        let focused = focused.filter(&filter);
                                        pages.insert(
                                            carrier,
                                            FrequencyPage {
                                                sampling: SamplingReport::from_rinex(&focused),
                                            },
                                        );
                                    }
                                }
                                pages
                            },
                        },
                    );
                }
                pages
            },
        }
    }
}

impl RenderHtml for Report {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                @ if let Some(rx) = &self.receiver {
                    table(class="table is-bordered") {
                        tr {
                            th(class="is-info") {
                                : "Receiver"
                            }
                            td {
                                : rx.to_inline_html()
                            }
                        }
                    }
                }
                @ if let Some(ant) = &self.antenna {
                    table(class="table is-bordered") {
                        tr {
                            th(class="is-info") {
                                : "Antenna"
                            }
                            td {
                                : ant.to_inline_html()
                            }
                        }
                    }
                }
                table(class="table is-bordered") {
                    th(class="is-info") {
                        : "Sampling"
                    }
                    td {
                        : self.sampling.to_inline_html()
                    }
                }
            }//table-container
        }
    }
}
