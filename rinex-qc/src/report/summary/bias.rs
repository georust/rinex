use maud::{html, Markup, Render};
use rinex::prelude::{GroundPosition, TimeScale};

use crate::prelude::{QcConfig, QcContext};
use crate::report::tooltipped;

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
