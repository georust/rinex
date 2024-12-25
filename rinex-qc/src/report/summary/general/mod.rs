use crate::{
    cfg::QcConfig,
    context::{meta::MetaData, QcContext},
    prelude::{html, Markup, Render},
};

mod nav;
mod obs;

use nav::QcNavigationSummary;
use obs::QcObservationsSummary;

/// [QcGeneralSummary] applies to the whole context
pub struct QcGeneralSummary {
    /// Configuration in use
    cfg: QcConfig,
    /// General observations data set summary
    observations: Option<QcObservationsSummary>,
    /// General navigation data set summary
    navigation: Option<QcNavigationSummary>,
}

impl QcGeneralSummary {
    pub fn new(ctx: &QcContext) -> Self {
        Self {
            cfg: ctx.cfg.clone(),
            observations: if ctx.has_observations() {
                Some(QcObservationsSummary::new(ctx))
            } else {
                None
            },
            navigation: if ctx.has_navigation_data() {
                Some(QcNavigationSummary::new(ctx))
            } else {
                None
            },
        }
    }
}

impl Render for QcGeneralSummary {
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
                            th class="is-info" {
                                "Observations"
                            }
                            td {
                                (observations.render())
                            }
                        }
                        @ if let Some(navigation) = &self.navigation {
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
