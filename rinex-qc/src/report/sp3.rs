use crate::report::shared::SamplingReport;
use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskFilter, MaskOperand, Preprocessing};
use sp3::prelude::{Constellation, SP3, SV};
use std::collections::HashMap;

//TODO
pub struct SP3Page {
    pub satellites: Vec<SV>,
    pub sampling: SamplingReport,
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
                            sampling: SamplingReport::from_sp3(&focused),
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
                }
            }
        }
    }
}
