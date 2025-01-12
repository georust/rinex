use cggtts::prelude::{CommonViewClass, Duration, Epoch, Track, SV};

use anise::prelude::Orbit;
use itertools::Itertools;

use crate::prelude::{html, Markup, Render};

#[derive(Default)]
pub struct Summary {
    is_first: bool,
    last_epoch: Epoch,
    first_epoch: Epoch,
    satellites: Vec<SV>,
    trk_duration: Duration,
    cv_class: CommonViewClass,
    initial_rx_orbit: Option<Orbit>,
}

impl Summary {
    /// Take new [Track] (just resolved) into account
    pub fn new_track(&mut self, trk: &Track) {
        if self.is_first {
            self.first_epoch = trk.epoch;
        } else {
            self.first_epoch = trk.epoch;
        }
    }
}

impl Render for Summary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Common View"
                            }
                            td {
                                (self.cv_class.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Track duration"
                            }
                            td {
                                (self.trk_duration.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Reference position"
                            }
                            // td {
                            //     (self.initial_rx_orbit.render())
                            // }
                        }
                        tr {
                            th class="is-info" {
                                "Satellites"
                            }
                            td {
                                (self.satellites.iter().join(", "))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "First Epoch"
                            }
                            td {
                                (self.first_epoch.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Last Epoch"
                            }
                            td {
                                (self.last_epoch.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Duration"
                            }
                            td {
                                ((self.first_epoch - self.last_epoch).to_string())
                            }
                        }
                    }
                }
            }
        }
    }
}
