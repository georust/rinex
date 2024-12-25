use crate::{
    context::{meta::MetaData, obs::ObservationUniqueId, QcContext},
    prelude::{html, Markup, Render, Rinex},
};

use std::str::FromStr;

enum Format {
    RINEx,
    GZipRINEx,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RINEx => f.write_str("RINEx"),
            Self::GZipRINEx => f.write_str("RINEx + gzip"),
        }
    }
}

pub struct QcNavigationSummary {
    pub agency: Option<String>,
}

impl QcNavigationSummary {
    pub fn new(ctx: &QcContext) -> Self {
        let nav_dataset = ctx.nav_dataset.as_ref().unwrap();

        Self {
            agency: nav_dataset.header.agency.clone(),
        }
    }
}

impl Render for QcNavigationSummary {
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
