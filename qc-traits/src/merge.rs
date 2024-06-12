//! Merge traits to extend data contexts
use thiserror::Error;

/// [Merge] specific Errors.
#[derive(Error, Debug)]
pub enum Error {
    /// When merging B into A, both types should match
    /// otherwise operation in invalid.
    #[error("file type mismatch")]
    FileTypeMismatch,
    /// Some file formats, to remain valid, require that
    /// B and A be expressed in the same Timescale to remain valid
    #[error("timescale mismatch")]
    TimescaleMismatch,
    /// Some file formats, to remain valid, require that coordinates
    /// from B and A be expressed in the same Reference Frame to remain valid
    #[error("reference frame mismatch")]
    ReferenceFrameMismatch,
    /// Some file formats, to remain valid, require that they are
    /// published by the same publisher/agency to be merged to into one another
    #[error("data provider (agency) mismatch")]
    DataProviderAgencyMismatch,
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
