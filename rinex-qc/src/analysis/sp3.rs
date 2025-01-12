use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
use std::collections::HashMap;

use sp3::prelude::{Constellation, SP3, SV};

use crate::{context::QcContext, //report::shared::SamplingReport
};

pub struct QcHighPrecisionPage {
    has_velocity: bool,
    has_clock: bool,
    has_clock_drift: bool,
    satellites: Vec<SV>,
    // sampling: SamplingReport,
}

impl Render for QcHighPrecisionPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "General"
                        }
                    }
                    tr {
                        th {
                            "Velocity"
                        }
                        td {
                            @ if self.has_velocity {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            } @ else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                    tr {
                        th {
                            "Clock offset"
                        }
                        td {
                            @ if self.has_clock {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            } @ else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                    tr {
                        th {
                            "Clock drift"
                        }
                        td {
                            @ if self.has_clock_drift {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            } @ else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Satellites"
                        }
                        td {
                            (self.satellites.iter().sorted().join(", "))
                        }
                    }
                }
            }
        }
    }
}

pub struct QcHighPrecisionNavigationReport {
    pub agency: String,
    pub version: String,
    pub coordinates_system: String,
    pub fit_method: String,
    pub constellation: String,
    pub time_scale: String,
    pub pages: HashMap<Constellation, QcHighPrecisionPage>,
}

impl QcHighPrecisionNavigationReport {
    pub fn new(sp3: &SP3) -> Self {
        Self {
            agency: sp3.agency.clone(),
            version: sp3.version.to_string(),
            fit_method: sp3.orbit_type.to_string(),
            coordinates_system: sp3.coord_system.clone(),
            time_scale: sp3.time_scale.to_string(),
            constellation: sp3.constellation.to_string(),
            pages: {
                let mut pages = HashMap::new();
                for constellation in sp3.constellation() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ConstellationItem(vec![constellation]),
                    );
                    let focused = sp3.filter(&filter);
                    //let epochs = focused.epoch().collect::<Vec<_>>();
                    let satellites = focused.sv().collect::<Vec<_>>();
                    pages.insert(
                        constellation,
                        QcHighPrecisionPage {
                            has_clock: focused.sv_clock().count() > 0,
                            // sampling: SamplingReport::from_sp3(&focused),
                            has_velocity: focused.sv_velocities().count() > 0,
                            has_clock_drift: focused.sv_clock_rate().count() > 0,
                            satellites,
                        },
                    );
                }
                pages
            },
        }
    }
}

impl Render for QcHighPrecisionNavigationReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            button aria-label="File revision" data-balloon-pos="right" {
                                "Revision"
                            }
                        }
                        td {
                            (self.version)
                        }
                    }
                    tr {
                        th {
                            button aria-label="Production Center" data-balloon-pos="right" {
                                "Agency"
                            }
                        }
                        td {
                            (self.agency.clone())
                        }
                    }
                    tr {
                        th {
                            button aria-label="Fitted constellations" data-balloon-pos="right" {
                                "Constellation"
                            }
                        }
                        td {
                            (self.constellation.clone())
                        }
                    }
                    tr {
                        th {
                            button aria-label="Timescale in which post-fit coordinates are expressed." data-balloon-pos="right" {
                                "Timescale"
                            }
                        }
                        td {
                            (self.time_scale.clone())
                        }
                    }
                    tr {
                        th {
                            button aria-label="Reference frame in which post-fit coordinates are expressed." data-balloon-pos="right" {
                                "Reference Frame"
                            }
                        }
                        td {
                            (self.coordinates_system.clone())
                        }
                    }
                    tr {
                        th {
                            button aria-label="Coordinates determination technique." data-balloon-pos="right" {
                                "Orbit FIT"
                            }
                        }
                        td {
                            (self.fit_method.clone())
                        }
                    }
                }//table
            }//table-container
            @for constell in self.pages.keys().sorted() {
                @if let Some(page) = self.pages.get(constell) {
                    div class="table-container is-page" id=(format!("sp3:{}", constell)) style="display:block" {
                        table class="table is-bordered" {
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

pub struct QcHighPrecisionNavigationReports {
    pub pages: HashMap<String, QcHighPrecisionNavigationReport>,
}

impl QcHighPrecisionNavigationReports {
    pub fn new(ctx: &QcContext) -> Self {
        let mut pages = HashMap::new();
        for (meta, sp3) in ctx.sp3_dataset.iter() {
            pages.insert(
                meta.name.to_string(),
                QcHighPrecisionNavigationReport::new(sp3),
            );
        }
        Self { pages }
    }
}
