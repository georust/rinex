use crate::{
    context::{meta::ObsMetaData, QcContext},
    prelude::{html, Render, Markup},
};

use rinex::prelude::Rinex;

pub struct QcBaseSummary {}

impl QcBaseSummary {
    pub fn new(ctx: &QcContext, meta: &ObsMetaData, rinex: &Rinex) -> Self {
        Self {}
    }
}

impl Render for QcBaseSummary {
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
