use crate::{cfg::QcConfig, context::QcContext};

pub(crate) mod nav;
pub(crate) mod obs;

use nav::QcNavigationSummary;
use obs::QcObservationsSummary;

/// [QcGeneralSummary] applies to the whole context
pub struct QcGeneralSummary {
    /// Configuration in use
    pub cfg: QcConfig,
    /// General observations data set summary
    pub observations: Option<QcObservationsSummary>,
    /// General navigation data set summary
    pub navigation: Option<QcNavigationSummary>,
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
