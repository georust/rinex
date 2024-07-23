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
                                "NAVI" // "Post processed navigation is not feasible."))
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Post processed navigation is feasible: Pseudo Range + BRDC and/or SP3" data-balloon-pos="up" {
                                    "NAVI"
                                }
                            }
                        }
                        td {
                            @if self.cpp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "CPP" // "CPP navigation compatible (dual freq. pseudo range)"))
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "CPP" // "Incompatible with CPP navigation: missing secondary frequency."))
                            }
                        }
                        td {
                            @if self.ppp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "PPP" // "PPP navigation compatible (dual freq. + phase range)"))
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "PPP" // "Incompatible with PPP navigation: missing secondary frequency or phase range."))
                            }
                        }
                        td {
                            @if self.ppp_ultra_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "PPP (Ultra)" // "PPP Ultra precise navigation"))
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "PPP (Ultra)" // "PPP Ultra incompatible: OBS + CLK should be synchronous in same timescale"))
                            }
                        }
                    }
                }
            }
        }
    }
}
