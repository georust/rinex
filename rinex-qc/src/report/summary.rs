use crate::{QcConfig, QcContext};
use rinex::prelude::TimeScale;

use qc_traits::html::*;

pub struct QcNavPostSummary {
    /// Navigation compatible
    nav_compatible: bool,
    /// PPP compatible
    ppp_compatible: bool,
    /// PPP ultra compatible
    ppp_ultra_compatible: bool,
}

impl QcNavPostSummary {
    pub fn new(context: &QcContext) -> Self {
        Self {
            nav_compatible: context.nav_compatible(),
            ppp_compatible: context.ppp_compatible(),
            ppp_ultra_compatible: context.ppp_ultra_compatible(),
        }
    }
}

impl RenderHtml for QcNavPostSummary {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table; style=\"margin-bottom: 20px\"") {
                thead {
                    tr {
                        td {
                            input(type="checkbox") {
                                : "NAVI"
                            }
                        }
                    }
                    tr {
                        td {
                            input(type="checkbox") {
                                : "PPP"
                            }
                        }
                    }
                    tr {
                        td {
                            input(type="checkbox") {
                                : "Ultra PPP"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// [QcSummary] is the lightest report form,
/// sort of a report introduction that will always be generated.
/// It only gives high level and quick description.
pub struct QcSummary {
    /// Configuration used
    cfg: QcConfig,
    /// Main timescale
    timescale: TimeScale,
    /// Post NAV summary
    nav_post: QcNavPostSummary,
}

impl QcSummary {
    pub fn new(context: &QcContext, cfg: &QcConfig) -> Self {
        Self {
            cfg: cfg.clone(),
            timescale: context.timescale(),
            nav_post: QcNavPostSummary::new(context),
        }
    }
}

impl RenderHtml for QcSummary {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table; style=\"margin-bottom: 20px\"") {
                tr {
                    td {
                        : self.cfg.to_inline_html()
                    }
                }
                tr {
                    td {
                        : "Timescale"
                    }
                    td {
                        : self.timescale.to_string()
                    }
                }
                tr {
                    td {
                        : self.nav_post.to_inline_html()
                    }
                }
            }
        }
    }
}
