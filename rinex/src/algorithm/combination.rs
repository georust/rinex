//! GNSS signal combination package
use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};

/// GNSS Combinations,
/// cf. <https://github.com/gwbres/rinex/blob/main/rinex-cli/doc/gnss-combination.md>.
#[derive(Debug, Clone, Copy)]
pub enum Combination {
    /// Geometry Free (Gf) combination cancels out geometric
    /// biases and leaves frequency dependent terms out,
    /// like the ionospheric delay
    GeometryFree,
    /// Wide Lane (WL) Phase combination
    WideLane,
    /// Narrow Lane (NL) Pseudo Range Combination
    NarrowLane,
    /// Melbourne-Wübbena (MW) combination
    MelbourneWubbena,
}

pub trait Combine {
    /// Form the combination on all available signals
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*;
    /// use rinex::processing::{Combination, Combine};
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///		.unwrap();
    /// let gf = rinex.combine<Combination::GeometryFree>;
    /// ```
    fn combine(
        &self,
        combination: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;

    // /// Form the combination for desired (lhs, reference) signals
    // fn combine_signals(&self, combination: Combination, signals: (Observable, Observable)) -> Option<HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>;
}
