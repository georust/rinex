//! RINEX Split operation
use crate::Epoch;
use thiserror::Error;

/// Split operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
}

pub trait Split<T> {
    /// Splits `Self` at desired epoch, 
    /// retaining |e(k) < epoch| ("before"), as left component,
    /// and |e(k) >= epoch| ("inclusive after"), as right component.
    /// Fails if self is not indexed by `Epoch`.
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error> where Self: Sized;
}
