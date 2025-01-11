use maud::{html, Markup, Render};

use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::Rinex,
};

use itertools::Itertools;

use rinex::prelude::Constellation;

use std::collections::HashMap;

struct QcNavConstellationSummary {
    brdc_strategy_compatible: bool,
    ppp_strategy_compatible: bool,
    ultra_ppp_strategy_compatible: bool,
}

impl Render for QcNavConstellationSummary {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tbody {
                    tr {
                        td {
                            @if self.brdc_strategy_compatible {
                                td {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                    button aria-label="Navigation using radio messages"
                                    data-balloon-pos="right" {
                                        "BRDC"
                                    }
                                }
                            } @ else {
                                td {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                    button aria-label="BRDC navigation needs Navigation RINEx"
                                    data-balloon-pos="right" {
                                        "BRDC"
                                    }
                                }
                            }
                        }
                        td {
                            @if self.ppp_strategy_compatible {
                                td {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                    button aria-label="PPP navigation using SP3"
                                    data-balloon-pos="right" {
                                        "PPP"
                                    }
                                }
                            } @ else {
                                td {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                    button aria-label="PPP navigation needs a matching SP3"
                                    data-balloon-pos="right" {
                                        "PPP"
                                    }
                                }
                            }
                        }
                        td {
                            @if self.ultra_ppp_strategy_compatible {
                                td {
                                    span class="icon" style="color:green" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                    button aria-label="Ultra PPP using synchronous Clock RINEx"
                                    data-balloon-pos="left" {
                                        "Ultra-PPP"
                                    }
                                }
                            } @ else {
                                td {
                                    span class="icon" style="color:red" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                    button aria-label="Ultra PPP navigation needs a synchronous Clock RINEx"
                                    data-balloon-pos="left" {
                                        "Ultra-PPP"
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

pub struct QcNaviSummary {
    html_id: String,
    tropo_model_optimization: bool,
    constellations_navi: HashMap<Constellation, QcNavConstellationSummary>,
}

impl QcNaviSummary {
    pub fn new(ctx: &QcContext, obs_meta: &ObsMetaData, rover: &Rinex) -> Self {
        Self {
            html_id: obs_meta.meta.name.to_string(),
            tropo_model_optimization: ctx.allows_troposphere_model_optimization(&obs_meta.meta),
            constellations_navi: {
                let mut constellations_sum = HashMap::new();
                for constellation in rover.constellations_iter() {
                    let nav_constellations = if let Some(rinex) = &ctx.nav_dataset {
                        rinex.constellations_iter().collect::<Vec<_>>()
                    } else {
                        vec![]
                    };

                    let brdc_strategy_compatible = nav_constellations.contains(&constellation);
                    let mut ppp_strategy_compatible = false;
                    let mut ultra_ppp_strategy_compatible = false;

                    #[cfg(feature = "sp3")]
                    if brdc_strategy_compatible {
                        // TODO SP3 support
                    }

                    let sum = QcNavConstellationSummary {
                        brdc_strategy_compatible,
                        ppp_strategy_compatible,
                        ultra_ppp_strategy_compatible,
                    };

                    constellations_sum.insert(constellation, sum);
                }
                constellations_sum
            },
        }
    }
}

impl Render for QcNaviSummary {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tbody {
                    tr {
                        @if self.tropo_model_optimization {
                            td {
                                span class="icon" style="color:green" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                                button aria-label="Troposphere bias model optimized with regionnal Meteo Data (RINEx)."
                                data-balloon-pos="center" {
                                    "Troposphere model optimization"
                                }
                            }
                        } @else {
                            td {
                                span class="icon" style="color:red" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                                button aria-label="Troposphere model optimization needs regionnal Meteo Data (RINEx)."
                                data-balloon-pos="center" {
                                    "Troposphere model optimization"
                                }
                            }
                        }

                        @ if !self.constellations_navi.is_empty() {
                            tr {
                                th class="is-info" {
                                    "Navigation Strategy"
                                }
                                // constellation selector
                                td {
                                    select id=(&self.html_id) onclick="onQcNaviSummarySelectionChanges()" {
                                        @ for constellation in self.constellations_navi.keys().sorted() {
                                            option value=(constellation.to_string()) {
                                                (constellation.to_string())
                                            }
                                        }
                                    }
                                }
                                // constellation pages
                                @ for (nth, (constellation, navi_sum)) in self.constellations_navi.iter().enumerate() {
                                    @ if nth == 0 {
                                        tr class="qc-navi-sum-selected" id=(constellation.to_string()) style="display:block" {
                                            td {
                                                (format!("{} compliancy", constellation))
                                            }
                                            td {
                                                (navi_sum.render())
                                            }
                                        }
                                    } @ else {
                                        tr class="qc-navi-sum-selected" id=(constellation.to_string()) style="display:none" {
                                            td {
                                                (format!("{} compliancy", constellation))
                                            }
                                            td {
                                                (navi_sum.render())
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
    }
}
