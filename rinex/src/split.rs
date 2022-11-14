use super::Epoch;
use thiserror::Error;

/// Split operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("this record type is not indexed by epoch")]
    NoEpochIteration,
}

pub trait Split<T> {
    /// Splits Self at desired epoch, 
    /// retaining |e(k) < epoch| ("before"), as left component,
    /// and |e(k) >= epoch| ("inclusive after"), as right component.
    /// Fails if self is not indexed by `Epoch`.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::epoch;
    /// use rinex::split::Split;
    /// let rnx = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let epoch = Epoch {
    ///     date: epoch::str2date("2021 01 01 0 1 00.0").unwrap(), // split after 3rd epoch (1sec into file)
    ///     flag: EpochFlag::Ok,
    /// };
    /// let (rnx_a, rnx_b) = rnx.split(epoch)
    ///     .unwrap();
    /// assert_eq!(rnx_a.epochs().len(), 2);
    /// assert_eq!(rnx_b.epochs().len(), rnx.epochs().len() - 2);
    /// ```
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error> where Self: Sized;
}
