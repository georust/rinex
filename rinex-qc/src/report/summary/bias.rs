use crate::prelude::QcContext;
use maud::{html, Markup, Render};
// use crate::report::tooltipped;
// use rinex::prelude::{GroundPosition, TimeScale};

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
                            button aria-label="Troposphere bias cancelling" data-balloon-pos="up" {
                                "Troposphere Bias"
                            }
                        }
                        @if self.tropo_bias_model_optimization {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Troposphere bias model can be optimized.
        Standard internal model is optimized with regional measurements" data-balloon-pos="up" {
                                    "Model optimization"
                                }
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Troposphere bias model cannot be optimized: missing Meteo IONEX.
        Bias modelling will solely rely on internal standard models." data-balloon-pos="up" {
                                    "Model optimization"
                                }
                            }
                        }
                    }
                    tr {
                        th {
                            button aria-label="Ionosphere bias cancelling" data-balloon-pos="up" {
                                "Ionosphere Bias"
                            }
                        }
                        @if self.iono_bias_model_optimization {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Ionosphere bias model optimized by IONEX measurement/prediction.
        This will not impact your solutions if direct cancellation is feasible." data-balloon-pos="up" {
                                    "Model optimization"
                                }
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Ionosphere bias model cannot be optimized: import a IONEX (special RINEX).
        This will not impact your solutions if direct cancellation is feasible." data-balloon-pos="up" {
                                    "Model optimization"
                                }
                            }
                        }
                        @if self.iono_bias_cancelling {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Direct IONOD cancellation by signal observation." data-balloon-pos="up" {
                                    "Cancelling"
                                }
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Direct IONOD cancellation is not feasible: missing secondary frequency." data-balloon-pos="up" {
                                    "Cancelling"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
