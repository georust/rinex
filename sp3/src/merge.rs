//! SP3 file merging operations.

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum MergeError {
    // #[error("data providers (agencies) should match, when merging two sp3")]
    // DataProvider,
    #[error("timescales should match when merging two sp3")]
    TimeScale,
    #[error("coords system (ref. frame) should match when merging two sp3")]
    CoordSystem,
}

pub trait Merge {
    /// Merge two SP3 files together: this introduces
    /// need Epoch and new data into self.
    /// We tolerate different sample rate, in this case
    /// self.epoch_interval becomes the lowest sample rate (pessimistic).
    /// File version is adjusted to newest revision.
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError>
    where
        Self: Sized;
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError>
    where
        Self: Sized;
}
