use crate::report::shared::SamplingReport;
use qc_traits::html::*;
use sp3::prelude::SP3;

pub struct SP3Report {
    pub name: String,
    pub agency: String,
    pub version: String,
    pub coord_system: String,
    pub orbit_fit: String,
    pub constellation: String,
    pub time_scale: String,
    pub sampling: SamplingReport,
}

impl SP3Report {
    pub fn new(sp3: &SP3) -> Self {
        Self {
            name: sp3.name.clone(),
            agency: sp3.agency.clone(),
            version: sp3.version.to_string(),
            coord_system: sp3.coord_system.clone(),
            orbit_fit: sp3.orbit_type.to_string(),
            time_scale: sp3.time_scale.to_string(),
            sampling: SamplingReport::from_sp3(sp3),
        }
    }
}

impl RenderHtml for SP3Report {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table {
                tr {
                    th {
                        : "Name"
                    }
                    td {
                        : self.name
                    }
                }
                tr {
                    th {
                        : "Agency"
                    }
                    td {
                        : self.agency
                    }
                }
                tr {
                    th {
                        : "Constellation"
                    }
                    td {
                        : self.constellation
                    }
                }
                tr {
                    th {
                        : "Timescale"
                    }
                    td {
                        : self.time_scale
                    }
                }
                tr {
                    th {
                        : "Reference Frame"
                    }
                    td {
                        : self.coord_system
                    }
                }
                tr {
                    th {
                        : "Orbit FIT"
                    }
                    td {
                        : self.orbit_fit
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
