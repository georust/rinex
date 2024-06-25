use maud::{html, Markup, Render};
use rinex::prelude::{GroundPosition, TimeScale};

use crate::prelude::{QcConfig, QcContext};
use crate::report::tooltipped;

pub struct QcNavPostSummary {
    /// Navigation compatible
    nav_compatible: bool,
    /// CPP compatible
    cpp_compatible: bool,
    /// PPP compatible
    ppp_compatible: bool,
    /// PPP ultra compatible
    ppp_ultra_compatible: bool,
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
                                (tooltipped("NAVI", "Navigation compatible context"))
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                (tooltipped("NAVI", "Navigation compatible context. Contains at least pseudo range observations along navigation data."))
                            }
                        }
                        td {
                            @if self.cpp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                (tooltipped("CPP", "CPP navigation compatible context. Dual frequency pseudo range observations."))
                            } @else {
                                span class="icon" style="color:red"{
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                (tooltipped("CPP", "CPP navigation compatible context. Dual frequency pseudo range observations."))
                            }
                        }
                        td {
                            @if self.ppp_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                (tooltipped("PPP", "PPP navigation compatible context. Dual frequency pseudo range + phase range observations."))
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                (tooltipped("PPP", "PPP navigation compatible context. Dual frequency pseudo range + phase range observations."))
                            }
                        }
                        td {
                            @if self.ppp_ultra_compatible {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                (tooltipped("PPP (Ultra)", "PPP Ultra precisision compatible. Dual frequency pseudo range + phase range observations and Clock RINEX without time interpolation."))
                            } @else {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                (tooltipped("PPP (Ultra)", "PPP Ultra precisision compatible. Dual frequency pseudo range + phase range observations and Clock RINEX without time interpolation."))
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct QcBiasSummary {
    iono_bias_cancelling: bool,
    iono_bias_model_optimization: bool,
    tropo_bias_model_optimization: bool,
}

impl QcBiasSummary {
    pub fn new(context: &QcContext) -> Self {
        Self {
            iono_bias_cancelling: context.cpp_compatible(),
            iono_bias_model_optimization: context.iono_bias_model_optimization(),
            tropo_bias_model_optimization: context.tropo_bias_model_optimization(),
        }
    }
}

impl Render for QcBiasSummary {
    fn render(&self) -> Markup {
        html! {
            table class="table" {
                tbody {
                    tr {
                        th {
                            "Troposphere Bias"
                        }
                        @if self.tropo_bias_model_optimization {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "Model optimization"
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "Model optimization"
                            }
                        }
                    }
                    tr {
                        th {
                            "Ionosphere Bias"
                        }
                        @if self.iono_bias_model_optimization {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "Model optimization"
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "Model optimization"
                            }
                        }
                        @if self.iono_bias_cancelling {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                "Cancelling"
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                "Cancelling"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// [QcSummary] is the lightest report form,
/// sort of a report introduction that will always be generated.
/// It only gives high level and quick description.
pub struct QcSummary {
    name: String,
    /// Configuration used
    cfg: QcConfig,
    /// Post NAV summary
    nav_post: QcNavPostSummary,
    /// Main timescale
    timescale: Option<TimeScale>,
    /// BIAS summary
    bias_sum: QcBiasSummary,
    /// reference position
    reference_position: Option<GroundPosition>,
}

impl QcSummary {
    pub fn new(context: &QcContext, cfg: &QcConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            name: context.name(),
            timescale: context.timescale(),
            bias_sum: QcBiasSummary::new(context),
            nav_post: QcNavPostSummary::new(context),
            reference_position: context.reference_position(),
        }
    }
}

impl Render for QcSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info is-bordered" {
                                (self.name.clone())
                            }
                        }
                        tr {
                            th {
                                "Timescale"
                            }
                            @if let Some(timescale) = self.timescale {
                                td {
                                    (timescale.to_string())
                                }
                            } @else {
                                td {
                                    "Not Applicable"
                                }
                            }
                        }
                        tr {
                            @if let Some(position) = self.cfg.manual_reference {
                                th {
                                    "(Manual) Reference position"
                                }
                                td {
                                    (position.render())
                                }
                            } @else if let Some(position) = self.reference_position {
                                th {
                                    "Reference position"
                                }
                                td {
                                    (position.render())
                                }
                            } @else {
                                th {
                                    "Reference position"
                                }
                                td {
                                    "None"
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Compliancy"
                            }
                            td {
                                (self.nav_post.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Bias"
                            }
                            td {
                                (self.bias_sum.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
