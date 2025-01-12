use crate::{
    cfg::{QcConfig, QcCustomRoverOpts, QcNaviOpts, QcPreferedSettings, QcReportOpts, QcSolutions},
    prelude::{html, Markup, QcHtmlReporting},
};

impl QcHtmlReporting for QcPreferedSettings {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Clock source"
                        }
                        td {
                            (self.clk_source.to_string())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Orbit source"
                        }
                        td {
                            (self.orbit_source.to_string())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Rovers sorting"
                        }
                        td {
                            (self.rovers_sorting.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcReportOpts {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Report Type"
                        }
                        td {
                            (self.report_type.to_string())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Signal combinations"
                        }
                        td {
                            (self.signal_combinations.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcCustomRoverOpts {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            "Reference position"
                        }
                        @if let Some(manual) = self.manual_rx_ecef_km {
                            td {
                                "Manual (User Defined)"
                            }
                            td {
                                (format!("{:.3E} km {:.3E} km {:.3E} km", manual.0, manual.1, manual.2))
                            }
                        } else {
                            td {
                                "RINEx"
                            }
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Prefered Rover"
                        }
                        td {
                            (self.prefered_rover.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcSolutions {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "PPP"
                        }
                        @ if self.ppp {
                            td {
                                button aria-label="PPP solutions attached to this report"
                                data-balloon-pos="right" {
                                    span class="icon" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                }
                            }
                        } @ else {
                            td {
                                button aria-label="PPP solutions not attached to this report"
                                data-balloon-pos="right" {
                                    span class="icon" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
                                }
                            }
                        }
                    }
                    tr {
                        th class="is-info" {
                            @ if self.cggtts {
                                button aria-label="CGGTTS solutions attached to this report"
                                data-balloon-pos="right" {
                                    "CGGTTS"
                                }
                            } @ else {
                                button aria-label="CGGTTS solutions not attached to this report"
                                data-balloon-pos="right" {
                                    "CGGTTS"
                                }
                            }
                        }
                        @ if self.cggtts {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            }
                        } @ else {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcNaviOpts {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            "Frame Model"
                        }
                        td {
                            (self.frame_model.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcConfig {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Reporting"
                        }
                        td {
                            (self.report.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Preference"
                        }
                        td {
                            (self.preference.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Navigation settings"
                        }
                        td {
                            (self.navi.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Rover settings"
                        }
                        td {
                            (self.rover.render())
                        }
                    }
                    @ if cfg!(feature = "nav") {
                        tr {
                            th class="is-info" {
                                "Solutions"
                            }
                            td {
                                (self.solutions.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
