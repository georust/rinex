//! Constellation pagination, during signals analysis

use std::collections::HashMap;

use crate::{
    report::{
        obs::{Physics, frequency::FrequencyPage},
        shared::SamplingReport,
    },
    prelude::Plot,
};

use rinex::prelude::{
    Rinex,
    Epoch,
    Constellation,
    obs::{SignalObservation, EpochFlag},
    SV,
};

/// Constellation dependent pagination
#[derive(Default)]
pub struct ConstellationPage {
    /// Satellites
    satellites: Vec<SV>,
    /// sampling
    sampling: SamplingReport,
    /// Signal dependent pagination
    frequency_pages: HashMap<String, FrequencyPage>,
}


impl ConstellationPage {

    /// Latch new data point in the current analysis
    pub fn new_symbol(&mut self, t: Epoch, flag: EpochFlag, signal: &SignalObservation) {


        let satellites = rinex.sv_iter().collect::<Vec<_>>();
        let sampling = SamplingReport::from_rinex(rinex);

        let mut frequency_pages = HashMap::<String, FrequencyPage>::new();

        // // browse all data points and add contributions to each frequency page
        // for (k, v) in rinex.signal_observations_iter() {
        //     if let Ok(carrier) = Carrier::from_observable(v.sv.constellation, &v.observable) {
        //         if let Ok(page) = frequency_pages.get_mut(&carrier) {

        //         } else {
        //             frequency_pages.insert(FrequencyPage::new());
        //         }
        //     }
        // }

        Self {
            satellites,
            sampling,
            frequency_pages,
        }
    }
}

impl Render for ConstellationPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th {
                                button aria-label="Pseudo Range single frequency navigation" data-balloon-pos="right" {
                                    "SPP Compatible"
                                }
                            }
                            td {
                                @if self.spp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                button aria-label="Pseudo Range dual frequency navigation" data-balloon-pos="right" {
                                    "CPP compatible"
                                }
                            }
                            td {
                                @if self.cpp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                }
                            }
                        }
                        tr {
                            th {
                                button aria-label="Dual frequency Pseudo + Phase Range navigation" data-balloon-pos="right" {
                                    "PPP compatible"
                                }
                            }
                            td {
                                @if self.ppp_compatible {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {};
                                    }
                                } @else {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {};
                                    }
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Observed Satellites" data-balloon-pos="right" {
                                    "Satellites"
                                }
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
                        tr {
                            th class="is-info" {
                                "Signals"
                            }
                            td {
                                (self.frequencies.keys().sorted().join(", "))
                            }
                        }
                        @for signal in self.frequencies.keys().sorted() {
                            @if let Some(page) = self.frequencies.get(signal) {
                                tr {
                                    th class="is-info" {
                                        (signal.to_string())
                                    }
                                    td id=(format!("page:obs:{}", signal)) class="page:obs:{}" style="display:block" {
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
}