use qc_traits::html::*;

use rinex::{
    hardware::{Antenna, Receiver},
    prelude::{Carrier, Constellation, Duration, Epoch},
};

use crate::report::shared::SamplingReport;

/// Frequency dependent pagination
pub struct FrequencyPage {
    /// Carrier
    pub carrier: Carrier,
    /// Loss of sight analysis
    pub gaps: HashMap<(Observable, Epoch), Duration>,
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

/// RINEX Observation Report
pub struct Report {
    pub antenna: Option<Antenna>,
    pub receiver: Option<Receiver>,
    pub sampling: SamplingReport,
    pub constellations: Vec<Constellation>,
    pub pages: HashMap<Constellation, ConstellationPage>,
}

impl Report {
    pub fn new(rinex: &Rinex) -> Self {
        Self {
            sampling: SamplingReport::from_rinex(rinex),
            receiver: if let Some(rcvr) = rinex.header.rcvr {
                Some(rcvr)
            } else {
                None
            },
            constellations: rinex.constellation().collect(),
            pages: {
                let mut pages = HashMap::<Constellation, ConstellationPage>::new();
                for constellation in rinex.constellation() {
                    let filter = Filter.mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = rinex.filter(&filter);
                    pages.insert(
                        constellation,
                        ConstellationPage {
                            doppler: focused.dopler().count() > 0,
                            spp_compatible: false,
                            cpp_compatible: false,
                            ppp_compatible: false,
                            pages: {
                                let mut pages = Vec::<FrequencyPage>::new();
                                for observable in focused.observable() {
                                    if let Ok(carrier) =
                                        Carrier::from_observable(observable, constellation)
                                    {
                                        let filter = Filter.mask(
                                            MaskOperand::Equals,
                                            FilterItem::ComplexItem(vec![observable.to_string()]),
                                        );
                                        let focused = focused.filter(&filter);
                                        pages.push(FrequencyPage {
                                            carrier,
                                            gaps: {
                                                let mut gaps =
                                                    HashMap::<(Observable, Epoch), Duration>::new();
                                                for (t, dur) in focused.data_gaps() {
                                                    gaps.insert((observable, t), dur);
                                                }
                                                gaps
                                            },
                                        });
                                    }
                                }
                            },
                        },
                    );
                }
            },
        }
    }
}

impl HtmlRender for Report {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table {
                th {
                    : "Antenna"
                }
                @ if let Some(ant) = self.antenna {
                    td {
                        : ant.to_inline_html()
                    }
                } else {
                    th {
                        : "Unknown"
                    }
                }
                th {
                    : "Receiver"
                }
                @ if let Some(rx) = self.receiver {
                    td {
                        : rx.to_inline_html()
                    }
                } else {
                    th {
                        : "Unknown"
                    }
                }
                th {
                    : "Constellations"
                }
                td {
                    : self.constellations.iter().join(",")
                }
                // create constellation dependent tab
                @ for (constellation, page) in self.pages.iter() {
                    th {
                        : constellation.to_string()
                    }
                    td {
                        : page.to_inline_html()
                    }
                }
            }
        }
    }
}
