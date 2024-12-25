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

pub struct QcObservationSummary {
    name: String,
    format: Format,
    designator: Option<ObservationUniqueId>,
}

impl QcObservationSummary {
    pub fn new(meta: MetaData, rinex: &Rinex) -> Self {
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
            designator: if let Some(unique_id) = meta.unique_id {
                let designator = ObservationUniqueId::from_str(&unique_id).unwrap();
                Some(designator)
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
                summaries.push(QcObservationSummary::new(meta.clone(), &rinex))
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
