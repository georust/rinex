use crate::prelude::{
    Config, Timescale, QcContext
};

pub struct QcNavPostSummary {
    /// Navigation compatible
    nav_compatible: bool,
    /// PPP compatible
    ppp_compatible: bool,
    /// PPP ultra compatible
    ppp_ultra_compatible: bool,
}

impl QcNavPostSummary {
    fn new(context: &QcContext) -> Self {
        Self {
            nav_compatible: context.nav_compatible(),
            ppp_compatible: context.ppp_compatible(),
            ppp_ultra_compatible: context.ppp_ultra_compatible(),
        }
    }
}

/// [QcSummary] is the lightest report form,
/// sort of a report introduction that will always be generated.
/// It only gives high level and quick description.
pub struct QcSummary {
    /// Configuration used
    cfg: Config,
    /// Main timescale
    timescale: Timescale,
    /// Post NAV summary
    nav_post: QcNavPostSummary,
}

impl QcSummary {
    fn new(context: &QcContext, cfg: &Config) -> Self {
        Self {
            cfg: cfg.clone(),
            timescale: TimeScale::default(), //TODO
            nav_post: QcNavPostSummary::new(context),        
        }
    }
}

