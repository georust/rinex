use gnss_rtk::prelude::PVTSolution;
use rinex::prelude::{nav::Orbit, Epoch, SV};

use crate::prelude::{html, Markup, Render};

#[derive(Default)]
pub struct Summary {
    is_first: bool,
    first_epoch: Epoch,
    last_epoch: Epoch,
    satellites: Vec<SV>,
    initial_rx_orbit: Option<Orbit>,
}

impl Summary {
    /// Latch new [PVTSolution] that has just been resolved
    pub fn new_solution(&mut self, t: Epoch, solution: PVTSolution) {
        if self.is_first {
            self.first_epoch = t;
        } else {
            self.last_epoch = t;
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
                                "Time frame"
                            }
                            td {
                                tr {
                                    td {
                                        "First"
                                    }
                                    td {
                                        "Last"
                                    }
                                }
                                tr {
                                    td {
                                        (self.first_epoch.to_string())
                                    }
                                    td {
                                        (self.first_epoch.to_string())
                                    }
                                }
                                tr {
                                    td {
                                        "Duration"
                                    }
                                    td {
                                        ((self.last_epoch - self.first_epoch).to_string())
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
