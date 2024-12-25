use crate::{
    context::{meta::MetaData, obs::ObservationUniqueId, QcContext},
    prelude::{html, Markup, Render, Rinex},
};

use itertools::Itertools;
use std::str::FromStr;

enum Format {
    RINEx,
    CRINEx,
    GZipRINEx,
    GZipCRINEx,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RINEx => f.write_str("RINEx"),
            Self::CRINEx => f.write_str("CRINEx"),
            Self::GZipRINEx => f.write_str("RINEx + gzip"),
            Self::GZipCRINEx => f.write_str("CRINEx + gzip"),
        }
    }
}

#[derive(Default)]
pub enum RxPositionSource {
    #[default]
    RINEx,
    UserDefined,
}

impl std::fmt::Display for RxPositionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RINEx => write!(f, "RINEx"),
            Self::UserDefined => write!(f, "User Defined"),
        }
    }
}

pub struct RxPosition {
    pub source: RxPositionSource,
    pub ecef_km: (f64, f64, f64),
    pub geodetic: (f64, f64, f64),
}

impl Render for RxPosition {
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

pub struct QcObservationSummary {
    name: String,
    format: Format,
    rx_position: Option<RxPosition>,
    designator: Option<ObservationUniqueId>,
}

impl QcObservationSummary {
    pub fn new(ctx: &QcContext, meta: MetaData, rinex: &Rinex) -> Self {
        Self {
            name: meta.name.to_string(),
            format: {
                let gzip = meta.extension.contains("gz");
                if rinex.header.is_crinex() {
                    if gzip {
                        Format::GZipCRINEx
                    } else {
                        Format::CRINEx
                    }
                } else {
                    if gzip {
                        Format::GZipRINEx
                    } else {
                        Format::RINEx
                    }
                }
            },
            designator: if let Some(unique_id) = &meta.unique_id {
                let designator = ObservationUniqueId::from_str(&unique_id).unwrap();
                Some(designator)
            } else {
                None
            },
            rx_position: if let Some(orbit) = ctx.meta_rx_orbit(&meta) {
                let pos_vel = orbit.to_cartesian_pos_vel();
                match orbit.latlongalt() {
                    Ok(geodetic) => Some(RxPosition {
                        source: RxPositionSource::RINEx,
                        ecef_km: (pos_vel[0], pos_vel[1], pos_vel[2]),
                        geodetic,
                    }),
                    Err(e) => {
                        error!("(anise): orbit.latlongalt: {}", e);
                        None
                    },
                }
            } else {
                None
            },
        }
    }
}

impl Render for QcObservationSummary {
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

pub struct QcObservationsSummary {
    nb_fileset: usize,
    summaries: Vec<QcObservationSummary>,
}

impl QcObservationsSummary {
    pub fn new(ctx: &QcContext) -> Self {
        let mut summaries = Vec::new();

        let metas = ctx.obs_dataset.keys().collect::<Vec<_>>();
        let nb_fileset = metas.len();

        for meta in metas.into_iter().unique() {
            if let Some(rinex) = ctx.obs_dataset.get(&meta) {
                summaries.push(QcObservationSummary::new(ctx, meta.clone(), &rinex))
            }
        }
        Self {
            nb_fileset,
            summaries,
        }
    }
}

impl Render for QcObservationsSummary {
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
