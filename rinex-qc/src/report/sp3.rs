use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
use std::collections::HashMap;

use sp3::prelude::{Constellation, SP3, SV};

use crate::report::shared::SamplingReport;

pub struct SP3Page {
    has_clock: bool,
    has_velocity: bool,
    has_clock_drift: bool,
    satellites: Vec<SV>,
    sampling: SamplingReport,
}

impl Render for SP3Page {
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
                            (self.has_velocity.to_string())
                        }
                    }
                    tr {
                        th {
                            "Clock offset"
                        }
                        td {
                            (self.has_clock.to_string())
                        }
                    }
                    tr {
                        th {
                            "Clock drift"
                        }
                        td {
                            (self.has_clock_drift.to_string())
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
                    tr {
                        th class="is-info" {
                            "Sampling"
                        }
                        td {
                            (self.sampling.render())
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
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:sp3" {
                span class="icon" {
                    i class="fa-solid fa-satellite" {}
                }
                (format!("High Precision Orbit (SP3{})", self.version))
            }
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
                    //let epochs = focused.epoch().collect::<Vec<_>>();
                    let satellites = focused.sv().collect::<Vec<_>>();
                    pages.insert(
                        constellation,
                        SP3Page {
                            has_clock: focused.sv_clock().count() > 0,
                            sampling: SamplingReport::from_sp3(&focused),
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

impl Render for SP3Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            button aria-label="File revision" data-balloon-pos="right" {
                                "File revision"
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
                            (self.coord_system.clone())
                        }
                    }
                    tr {
                        th {
                            button aria-label="Coordinates determination technique." data-balloon-pos="right" {
                                "Orbit FIT"
                            }
                        }
                        td {
                            (self.orbit_fit.clone())
                        }
                    }
                    tr {
                        th {
                            "Sampling"
                        }
                        td {
                            (self.sampling.render())
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
