//! Observations reporting

mod page;
use page::ObservationPage;

use crate::{
    cfg::QcConfig,
    context::{meta::MetaData, obs::ObservationUniqueId, QcContext},
    prelude::{html, Markup, Render, Rinex},
};

use std::{collections::HashMap, str::FromStr};

pub struct QcObservationsReport {
    pub rovers: Vec<String>,
    pub bases: Vec<String>,
    pub rover_pages: HashMap<String, ObservationPage>,
    pub base_pages: HashMap<String, ObservationPage>,
}

impl QcObservationsReport {
    pub fn new(ctx: &QcContext) -> Self {
        let mut rovers = Vec::new();
        let mut bases = Vec::new();
        let mut rover_pages = HashMap::new();
        let mut base_pages = HashMap::new();
        Self {
            rovers,
            bases,
            rover_pages,
            base_pages,
        }
    }
}

impl Render for QcObservationsReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Rovers"
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Bases"
                            }
                        }
                    }
                }
            }
        }
    }
}
