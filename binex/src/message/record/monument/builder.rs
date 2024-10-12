use hifitime::Epoch;

use super::{MonumentGeoFrame, MonumentGeoMetadata, MonumentGeoRecord};

pub struct MonumentGeoBuilder;

impl MonumentGeoBuilder {
    /// Builds new [MonumentGeoRecord] (default)
    /// which you can then customize.
    pub fn new(t: Epoch, meta: MonumentGeoMetadata) -> MonumentGeoRecord {
        MonumentGeoRecord {
            epoch: t,
            source_meta: meta,
            frames: Default::default(),
        }
    }
}
