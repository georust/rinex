//! RINEX Split operation
use crate::Epoch;
use thiserror::Error;
use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};
use std::hash::Hash;

/// Split operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
    #[error("epoch is too early")]
    EpochTooEarly,
    #[error("epoch is too late")]
    EpochTooLate,
}

pub trait Split<T> {
    /// Splits `Self` at desired epoch
    fn split_at_epoch(&self, epoch: Epoch) -> Result<(Self, Self), Error> where Self: Sized;
    //fn split(&self, rhs: &T) -> Result<(Self, Self), Error> where Self: Sized;
    //fn split_mut(&mut self, rhs: &T) -> Result<(), Error>;
}
