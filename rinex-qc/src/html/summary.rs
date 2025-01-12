use crate::{
    analysis::summary::{
        base::QcBaseSummary,
        general::{
            nav::QcNavigationSummary,
            obs::{QcObservationSummary, QcObservationsSummary, RxPosition},
            QcGeneralSummary,
        },
        rover::{
            navi::{QcNavConstellationSummary, QcNaviSummary},
            QcRoverSummary,
        },
        QcSummary,
    },
    context::obs::ObservationUniqueId,
    prelude::{html, Markup, QcHtmlReporting},
};

use itertools::Itertools;

impl QcHtmlReporting for QcBaseSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcRoverSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                button aria-label="NB: summary report does not account for the time frame,
only files & constellations relationship.\n
Use the timeframe analysis to actually confirm the summary report"
                                data-balloon-pos="right" {
                                    b {
                                        "Compliancy"
                                    }
                                }
                            }
                            td {
                                (self.navi.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcNaviSummary {
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
                                        tr id=(constellation.to_string()) style="display:block" {
                                            td {
                                                (format!("{} compliancy", constellation))
                                            }
                                            td {
                                                (navi_sum.render())
                                            }
                                        }
                                    } @ else {
                                        tr id=(constellation.to_string()) style="display:none" {
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

impl QcHtmlReporting for QcSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            td {
                                (self.general.render())
                            }
                        }
                        tr {
                            @ if self.bases_sum.len() == 0 {
                                @ if self.rovers_sum.len() == 1 {
                                    th class="is-info" {
                                        "Rover"
                                    }
                                } @ else {
                                    th class="is-info" {
                                        "Rovers"
                                    }
                                }
                            }
                        }
                        @ for (_, rover) in self.rovers_sum.iter() {
                            tr {
                                td {
                                    (rover.render())
                                }
                            }
                        }

                        @ if !self.bases_sum.is_empty() {
                            tr {
                                td {
                                    @ if self.bases_sum.len() == 1 {
                                        th class="is-info" {
                                            "Base Station"
                                        }
                                    } @ else {
                                        th class="is-info" {
                                            "Base Stations"
                                        }
                                    }
                                }
                            }
                            @ for (_, base) in self.bases_sum.iter() {
                                tr {
                                    td {
                                        (base.render())
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

impl QcHtmlReporting for QcObservationSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Name"
                            }
                            td {
                                (self.name)
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Format"
                            }
                            td {
                                (self.format)
                            }
                        }
                        @ if self.designator.is_none() {
                            th class="is-warning" {
                                "No designator"
                            }
                        }
                        @ if let Some(designator) = &self.designator {
                            th class="is-info" {
                                "Designator"
                            }
                            td {
                                @ match designator {
                                    ObservationUniqueId::Receiver(_) => {
                                        b {
                                            "GNSS RX "
                                        }
                                        (designator.to_string())
                                    },
                                    ObservationUniqueId::Antenna(_) => {
                                        b {
                                            "RX Antenna "
                                        }
                                        (designator.to_string())
                                    },
                                    ObservationUniqueId::GeodeticMarker(_) => {
                                        b {
                                            "Geodetic Marker "
                                        }
                                        (designator.to_string())
                                    },
                                    ObservationUniqueId::Operator(_) => {
                                        b {
                                            "Operator "
                                        }
                                        (designator.to_string())
                                    },
                                }
                            }
                        }
                        @ if let Some(rx_position) = &self.rx_position {
                            tr {
                                th class="is-info" {
                                    "Reference position"
                                }
                                td {
                                    (rx_position.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcObservationsSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Number fileset"
                            }
                            td {
                                (self.nb_fileset)
                            }
                        }
                        @ for summary in self.summaries.iter() {
                            tr {
                                td {
                                    (summary.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for RxPosition {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Source"
                            }
                            td {
                                (self.source.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "ECEF (km)"
                            }
                            td {
                                (format!("{:.3e}", self.ecef_km.0))
                            }
                            td {
                                (format!("{:.3e}", self.ecef_km.1))
                            }
                            td {
                                (format!("{:.3e}", self.ecef_km.2))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "GEO (ddeg)"
                            }
                            td {
                                (format!("{:.3e}", self.geodetic.0))
                            }
                            td {
                                (format!("{:.3e}", self.geodetic.1))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Altitude (above sea)"
                            }
                            td {
                                (format!("{:.3e}", self.geodetic.2))
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcNavConstellationSummary {
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

impl QcHtmlReporting for QcGeneralSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "QC Settings"
                            }
                            td {
                                (self.cfg.render())
                            }
                        }
                        @ if let Some(observations) = &self.observations {
                            tr {
                                th class="is-info" {
                                    "Observations"
                                }
                                td {
                                    (observations.render())
                                }
                            }
                        }
                        @ if let Some(navigation) = &self.navigation {
                            tr {
                                th class="is-info" {
                                    "Navigation"
                                }
                                td {
                                    (navigation.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl QcHtmlReporting for QcNavigationSummary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            @ if let Some(agency) = &self.agency {
                                th class="is-info" {
                                    "Agency"
                                }
                                td {
                                    (agency)
                                }
                            }
                            @ if self.agency.is_none() {
                                th class="is-warning" {
                                    "Unknown agency"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
