use maud::{html, Markup, Render};
use rinex::prelude::{nav::Orbit, TimeScale};

use crate::prelude::{QcConfig, QcContext};

mod nav_post;
use nav_post::QcNavPostSummary;

mod bias;
use bias::QcBiasSummary;

/// [QcSummary] is the lightest report form,
/// sort of a report introduction that will always be generated.
/// It only gives high level and quick description.
pub struct QcSummary {
    name: String,
    /// Configuration used
    cfg: QcConfig,
    /// NAVI summary
    pub navi: QcNavPostSummary,
    /// Main timescale
    timescale: Option<TimeScale>,
    /// BIAS summary
    bias_sum: QcBiasSummary,
    /// reference position
    reference_position: Option<Orbit>,
}

impl QcSummary {
    pub fn new(context: &QcContext, cfg: &QcConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            name: context.name(),
            timescale: context.timescale(),
            bias_sum: QcBiasSummary::new(context),
            navi: QcNavPostSummary::new(context),
            reference_position: context.reference_rx_orbit(),
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
                                button aria-label="Timescale in which samples observation are expressed.
        Navigation solutions are expressed in this timescale by default." data-balloon-pos="right" {
                                    "Timescale"
                                }
                            }
                            @if let Some(timescale) = self.timescale {
                                td {
                                    (timescale.to_string())
                                }
                            } @else {
                                td {
                                    button aria-label="This dataset is not a timeserie." data-balloon-pos="up" {
                                        "Not Applicable"
                                    }
                                }
                            }
                        }
                        tr {
                            @if let Some(orbit) = self.cfg.manual_rx_orbit {
                                @match orbit.latlongalt() {
                                    Ok((lat_ddeg, long_ddeg, alt_km)) => {
                                        th {
                                            button aria-label="RX reference position" data-balloon-pos="up" {
                                                "(Manual) Reference position"
                                            }
                                        }
                                        td {
                                            button aria-label="Manually defined" data-balloon-pos="up" {
                                                "TODO"
                                            }
                                        }
                                    },
                                    Err(e) => {
                                        th {
                                            button aria-label="RX reference position" data-balloon-pos="up" {
                                                "(Manual) Reference position"
                                            }
                                        }
                                        td {
                                            button aria-label="Manually defined" data-balloon-pos="up" {
                                                "Invalid"
                                            }
                                        }

                                    },
                                }
                            } @else if let Some(orbit) = self.reference_position {
                                @match orbit.latlongalt() {
                                    Ok(latlongalt) => {
                                        th {
                                            button aria-label="RX reference position" data-balloon-pos="up" {
                                                "Reference position"
                                            }
                                        }
                                        td {
                                            button aria-label="Parsed from RINEX" data-balloon-pos="up" {
                                                "TODO"
                                            }
                                        }
                                    },
                                    Err(e) => {

                                        th {
                                            button aria-label="RX reference position" data-balloon-pos="up" {
                                                "Reference position"
                                            }
                                        }
                                        td {
                                            button aria-label="Parsed from RINEX" data-balloon-pos="up" {
                                                "TODO"
                                            }
                                        }
                                    },
                                }
                            } @else {
                                th {
                                    button aria-label="Ground based reference position" data-balloon-pos="up" {
                                        "Reference position"
                                    }
                                }
                                td {
                                    button aria-label="Compass projection is disabled.
Most navigation geometric/attibutes filter cannot apply.
Initial survey/guess is implied." data-balloon-pos="up" {
                                        "None"
                                    }
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Context / Dataset compliancy" data-balloon-pos="right" {
                                    "Compliancy"
                                }
                            }
                            td {
                                (self.navi.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Physical and Environmental bias analysis & cancellation capabilities" data-balloon-pos="right" {
                                    "Bias"
                                }
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
