use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskFilter, MaskOperand, Preprocessing};

use std::collections::HashMap;

use rinex::{
    carrier::Carrier,
    hardware::{Antenna, Receiver},
    prelude::{Constellation, Duration, Epoch, Observable, Rinex},
};

use crate::report::shared::SamplingReport;

/// Frequency dependent pagination
pub struct FrequencyPage {
    /// Carrier
    pub carrier: Carrier,
    /// Loss of sight analysis
    pub gaps: HashMap<(Observable, Epoch), Duration>,
}

impl RenderHtml for FrequencyPage {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
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
    pub pages: Vec<FrequencyPage>,
}

impl RenderHtml for ConstellationPage {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
    }
}

/// RINEX Observation Report
pub struct Report {
    pub antenna: Option<Antenna>,
    pub receiver: Option<Receiver>,
    pub sampling: SamplingReport,
    pub pages: HashMap<Constellation, ConstellationPage>,
}

impl Report {
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
                                let mut pages = Vec::<FrequencyPage>::new();
                                for observable in focused.observable() {
                                    if let Ok(carrier) =
                                        Carrier::from_observable(constellation, observable)
                                    {
                                        let filter = Filter::mask(
                                            MaskOperand::Equals,
                                            FilterItem::ComplexItem(vec![observable.to_string()]),
                                        );
                                        let focused = focused.filter(&filter);
                                        pages.push(FrequencyPage {
                                            carrier,
                                            gaps: {
                                                let mut gaps =
                                                    HashMap::<(Observable, Epoch), Duration>::new();
                                                for (t, dur) in focused.data_gaps(None) {
                                                    // gaps.insert((observable, t), dur);
                                                }
                                                gaps
                                            },
                                        });
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
                table(class="table is-bordered") {
                    @ if let Some(ant) = &self.antenna {
                        td {
                            : ant.to_inline_html()
                        }
                    }
                }
            }//table-container
            div(class="table-container") {
                table(class="table is-bordered") {
                    @ if let Some(rx) = &self.receiver {
                        td {
                            : rx.to_inline_html()
                        }
                    }
                }
            }
            div(class="table-container") {
                : self.sampling.to_inline_html()
            }
        }
    }
}
