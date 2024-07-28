use crate::prelude::QcContext;
use maud::{html, Markup, Render};
//use rinex::prelude::{GroundPosition, TimeScale};

pub struct QcNavPostSummary {
    /// Navigation compatible
    pub nav_compatible: bool,
    /// CPP compatible
    pub cpp_compatible: bool,
    /// PPP compatible
    pub ppp_compatible: bool,
    /// PPP ultra compatible
    pub ppp_ultra_compatible: bool,
}

impl QcNavPostSummary {
    pub fn new(context: &QcContext) -> Self {
        Self {
            nav_compatible: context.nav_compatible(),
            cpp_compatible: context.cpp_compatible(),
            ppp_compatible: context.ppp_compatible(),
            ppp_ultra_compatible: context.ppp_ultra_compatible(),
        }
    }
}

impl Render for QcNavPostSummary {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tbody {
                    tr {
                        td {
                            @if self.nav_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Post processed navigation is feasible" data-balloon-pos="up" {
                                    "NAVI"
                                }
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Post processed navigation is not feasible. You must provide at least Pseudo Range + NAV (BRDC) RINEX and SP3 if desired." data-balloon-pos="up" {
                                    "NAVI"
                                }
                            }
                        }
                        td {
                            @if self.cpp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Code Based Precise Positioning. Direct IONOD cancelling." data-balloon-pos="up" {
                                    "CPP"
                                }
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Code Based Precise Positioning not feasible. Missing secondary frequency." data-balloon-pos="up" {
                                    "CPP"
                                }
                            }
                        }
                        td {
                            @if self.ppp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Precise Point Positioning is feasible. Dual PR+PH navigation with direct IONOD cancelling." data-balloon-pos="up" {
                                    "PPP"
                                }
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Precise Point Positioning is not feasible. Missing Phase Data and/or secondary frequency and/or SP3 and/or CLK RINEX." data-balloon-pos="up" {
                                    "PPP"
                                }
                            }
                        }
                        td {
                            @if self.ppp_ultra_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Ultimate PPP: CLK RINEX synchronous to OBS RINEX" data-balloon-pos="up" {
                                    "PPP (Ultra)"
                                }
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Ultimate PPP: CLK RINEX is not synchronous to OBS RINEX" data-balloon-pos="up" {
                                    "PPP (Ultra)"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
