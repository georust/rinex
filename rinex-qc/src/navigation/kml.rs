use crate::{
    context::{meta::MetaData, QcContext},
    QcError,
};

use gnss_rtk::prelude::Config;
use kml::KmlDocument;

impl QcContext {
    /// Collect all Navigation solutions and wrap them
    /// in a [KmlDocument]. You should only use this if you
    /// only intend to use the PVT solutions in KML format,
    /// otherwise it is much more efficient to format the tracks
    /// yourself, from previously obtained solutions: you only
    /// want to solve solutions once.
    pub fn kml_track_solutions(&self, cfg: Config, meta: MetaData) -> Result<KmlDocument, QcError> {
        let mut solver = self.nav_pvt_solver(cfg, &meta, None)?;
        Ok(KmlDocument::default())
    }
}
