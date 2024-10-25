//! Merge trait to extend datasets
use thiserror::Error;

#[cfg(docsrs)]
use hifitime::TimeScale;

/// [Merge] specific Errors.
#[derive(Error, Debug)]
pub enum Error {
    /// You can only [Merge] two compatible files toghether.
    #[error("file type mismatch")]
    FileTypeMismatch,
    /// Depending on file format, [Merge] may require that A & B be expressed
    /// in the same [TimeScale]
    #[error("timescale mismatch")]
    TimescaleMismatch,
    /// Depending on file format, [Merge] may require that A and B be expressed
    /// in the same reference coordinates system.
    #[error("reference frame mismatch")]
    ReferenceFrameMismatch,
    /// Depending on file format, [Merge] may require that A and B were produced
    /// by the same data provider.
    #[error("data provider mismatch")]
    DataProviderMismatch,
    /// Some file formats may require to have strictly the same dimensions
    /// for [Merge] to be feasible.
    #[error("dimensions mismatch")]
    DimensionMismatch,
    /// Other error that happend during [Merge] operation
    #[error("other error")]
    Other,
}

/// Merge Trait is impleted to extend Data Contexts.
pub trait Merge {
    /// Merge "rhs" dataset into self, to form extend dataset.
    /// We use this for example to extend 24h RINEX to 1week RINEX.
    /// When merging File A and B types must match otherwise operation is invalid.
    fn merge(&self, rhs: &Self) -> Result<Self, Error>
    where
        Self: Sized;
    /// [Self::merge] mutable implementation.
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), Error>;
}
