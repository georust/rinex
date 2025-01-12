use crate::{
    analysis::obs::{
        clock::ClockAnalysis, QcBasesObservationsAnalysis, QcObservationsAnalysis,
        QcRoversObservationAnalysis,
    },
    prelude::{html, MarkerSymbol, Markup, Mode, Plot, QcHtmlReporting},
};

pub struct ClockAnalysisHtml {
    clock_offset: Plot,
    clock_drift: Plot,
}

impl ClockAnalysis {
    // Convert to visual (ready to render)
    #[cfg(feature = "html")]
    fn to_html(&self, html_id: &str) -> ClockAnalysisHtml {
        let offset_label = format!("{}_clock_offset", html_id);
        let drift_label = format!("{}_clock_drift", html_id);

        let mut clock_offset = Plot::timedomain_plot(&offset_label, "Clock offset", "[s]", true);
        let offset_x = self.clock_offset_s.iter().map(|k| k.0).collect::<Vec<_>>();
        let offset_y = self.clock_offset_s.iter().map(|k| k.1).collect::<Vec<_>>();

        let trace = Plot::timedomain_chart(
            &offset_label,
            Mode::LinesMarkers,
            MarkerSymbol::Cross,
            &offset_x,
            offset_y,
            true,
        );

        clock_offset.add_trace(trace);

        let mut clock_drift = Plot::timedomain_plot(&drift_label, "Clock drift", "[s.s⁻¹]", true);
        let drift_x = self.clock_drift_s_s.iter().map(|k| k.0).collect::<Vec<_>>();
        let drift_y = self.clock_drift_s_s.iter().map(|k| k.1).collect::<Vec<_>>();

        let trace = Plot::timedomain_chart(
            &drift_label,
            Mode::LinesMarkers,
            MarkerSymbol::Cross,
            &drift_x,
            drift_y,
            true,
        );

        clock_drift.add_trace(trace);

        ClockAnalysisHtml {
            clock_offset,
            clock_drift,
        }
    }
}

impl QcHtmlReporting for ClockAnalysisHtml {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Clock offset"
                        }
                        td {
                            (self.clock_offset.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Clock drift"
                        }
                        td {
                            (self.clock_drift.render())
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcBasesObservationsAnalysis {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" id="qc-base-observations" style="display:none" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Base Stations"
                            }
                            // td {
                            //     select id="qc-base-obs-selector" onclick="onQcBaseObsSelection()" {
                            //         @ for base in self.reports.keys() {
                            //             option value=(base.name) {}
                            //         }
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcRoversObservationAnalysis {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" id="qc-rover-observations" style="display:none" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Rovers"
                            }
                            // td {
                            //     select id="qc-rover-obs-selector" onclick="onQcRoverObsSelection()" {
                            //         @ for rover in self.reports.keys() {
                            //             option value=(rover.name) {}
                            //         }
                            //     }
                            // }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcObservationsAnalysis {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                @if let Some(rx) = &self.receiver {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "Receiver"
                            }
                            td {
                                (rx.render())
                            }
                        }
                    }
                }
                @if let Some(ant) = &self.antenna {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "Antenna"
                            }
                            td {
                                 (ant.render())
                            }
                        }
                    }
                }
                @ if let Some(clk_analysis) = &self.clock_analysis {
                    table class="table is-bordered" {
                        tr {
                            th class="is-info" {
                                "RX Clock"
                            }
                            td {
                                (clk_analysis.to_html(&self.id).render())
                            }
                        }
                    }
                }
            }//table-container
        }
    }
}
