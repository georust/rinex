//! Observations reporting

mod page;
use page::ObservationPage;

use crate::{
    cfg::QcConfig,
    context::{meta::MetaData, obs::ObservationUniqueId, QcContext},
    prelude::{html, Markup, Rinex},
};

use std::str::FromStr;

pub struct QcObservationReportPage {
    pub designated_rover: bool,
    pub name: String,
    pub unique_id: Option<ObservationUniqueId>,
    pub content: ObservationPage,
}

impl QcObservationReportPage {
    pub fn new(meta: &MetaData, rinex: &Rinex) -> Self {
        Self {
            designated_rover: false,
            unique_id: if let Some(unique_id) = &meta.unique_id {
                Some(ObservationUniqueId::from_str(&unique_id).unwrap())
            } else {
                None
            },
            name: meta.name.to_string(),
            content: ObservationPage::new(rinex),
        }
    }

    pub fn html_menu_item(&self) -> Markup {
        html! {
            li {
                @ for (k, v) in self.content.constellations.iter() {
                    a id="menu:observations" {
                        span class="icon" {
                            i class="fa-solid fa-tower-broadcast" {}
                        }
                        (self.name)
                    }
                }
            }
        }
    }
}

pub struct QcObservationsReport {
    pub pages: Vec<QcObservationReportPage>,
}

impl QcObservationsReport {
    pub fn new(ctx: &QcContext) -> Self {
        let mut pages = Vec::new();
        Self { pages }
    }

    /// Provide easy browsing for HTML rendition
    pub fn html_menu_bar(&self) -> Markup {
        html! {
            ul class="menu-list" {
                @for page in self.pages.iter() {
                    li {
                        a {
                            span class="icon" {
                                i {}
                            }
                            @if let Some(unique_id) = &page.unique_id {
                                (unique_id.to_string())
                            } else {
                                "Unknown"
                            }
                        }
                    }
                }
            }
        }
    }
}
