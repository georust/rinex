//! RINEX Split operation
use crate::Epoch;
use thiserror::Error;
use std::cmp::{PartialEq, Eq};
use std::hash::Hash;

/// Split operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
}

pub trait Split<T> {
    /// Splits `Self` at desired epoch, retaining |e(k) <= epoch| as left component,
    /// and |e(k) > epoch| as right component.
    /// Fails if self is not indexed by `Epoch`.
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error> where Self: Sized;
}
