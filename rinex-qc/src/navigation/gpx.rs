use crate::{
    context::{meta::MetaData, QcContext},
    QcError,
};

use gnss_rtk::prelude::Config;
use gpx::Gpx;

impl QcContext {
    /// Collect all Navigation solutions and wrap them
    /// in as a GPX track. You should only use this if you
    /// only intend to use the PVT solutions in GPX format,
    /// otherwise it is much more efficient to format the tracks
    /// yourself, from previously obtained solutions: you only
    /// want to solve solutions once.
    pub fn gpx_track_solutions(&self, cfg: Config, meta: MetaData) -> Result<Gpx, QcError> {
        let mut solver = self.nav_pvt_solver(cfg, &meta, None)?;
        Ok(Gpx::default())
    }
}
