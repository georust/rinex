use maud::{html, Markup, Render};

use crate::{context::QcContext, prelude::Rinex};

struct QcNavConstellationSummary {
    iono_cancelling: bool,
    nav_compatible: bool,
    ppp_compatible: bool,
}

struct QcNavSummary {
    tropo_optimization: bool,
    constell_summary: HashMap<Constellation, QcNavConstellationSummary>,
}

impl QcNavPostSummary {
    pub fn new(context: &QcContext, rover: &Rinex) -> Self {}
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
                                button aria-label="Post processed navigation is feasible" data-balloon-pos="bottom" {
                                    "NAVI"
                                }
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Post processed navigation is not feasible. You must provide at least Pseudo Range + NAV (BRDC) RINEX and SP3 if desired." data-balloon-pos="bottom" {
                                    "NAVI"
                                }
                            }
                        }
                        td {
                            @if self.cpp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Code Based Precise Positioning. Direct IONOD cancelling." data-balloon-pos="bottom" {
                                    "CPP"
                                }
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Code Based Precise Positioning not feasible. Missing secondary frequency." data-balloon-pos="bottom" {
                                    "CPP"
                                }
                            }
                        }
                        td {
                            @if self.ppp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Precise Point Positioning is feasible. Dual PR+PH navigation with direct IONOD cancelling." data-balloon-pos="bottom" {
                                    "PPP"
                                }
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Precise Point Positioning is not feasible. Missing Phase Data and/or secondary frequency and/or SP3 and/or CLK RINEX." data-balloon-pos="bottom" {
                                    "PPP"
                                }
                            }
                        }
                        td {
                            @if self.ppp_ultra_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Ultimate PPP: CLK RINEX synchronous to OBS RINEX" data-balloon-pos="bottom" {
                                    "PPP (Ultra)"
                                }
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Ultimate PPP: CLK RINEX is not synchronous to OBS RINEX" data-balloon-pos="bottom" {
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
