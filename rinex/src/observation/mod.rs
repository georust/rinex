//! Observation RINEX module
mod clock;
mod flag;
mod formatting; // formatter
mod header;
mod lli;
mod parsing; // parser
mod rinex; // high level methods
mod signal;
mod snr;

#[cfg(feature = "processing")]
pub(crate) mod mask; // mask Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod decim; // decim Trait implementation

#[cfg(feature = "processing")]
pub(crate) mod repair; // repair Trait implementation

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use clock::ClockObservation;
pub use flag::EpochFlag;
pub use header::HeaderFields;
pub use lli::LliFlags;
pub use signal::SignalObservation;
pub use snr::SNR;

pub(crate) use formatting::format;
pub(crate) use parsing::{is_new_epoch, parse_epoch};

#[cfg(docsrs)]
use crate::Bibliography;

#[cfg(test)]
pub use formatting::fmt_observations;

use std::collections::BTreeMap;

use crate::prelude::Epoch;

/// [Observations] describes all the content an Observation Epoch
/// indexed by [ObsKey] may contain.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Observations {
    /// Local [ClockObservation] may exist, depending on receiver
    /// capabilities and production context.
    /// It describes the receiver state with respect to the GNSS [TimeScale] defined
    /// in [Header].
    pub clock: Option<ClockObservation>,
    /// List of [SignalObservation]s.
    pub signals: Vec<SignalObservation>,
}

impl Default for Observations {
    fn default() -> Self {
        Self {
            clock: None,
            signals: Vec::with_capacity(16),
        }
    }
}

impl Observations {
    /// Define [Observations] with Clock offset [s] observed at [Epoch]
    pub fn with_clock_offset_s(&self, timeof_obs: Epoch, offset_s: f64) -> Self {
        let mut s = self.clone();
        if let Some(ref mut clock) = s.clock {
            clock.set_offset_s(timeof_obs, offset_s);
        } else {
            s.clock = Some(ClockObservation::default().with_offset_s(timeof_obs, offset_s));
        }
        s
    }
    /// Define [Observations] with [ClockObservation]
    pub fn with_clock_observation(&self, clock: ClockObservation) -> Self {
        let mut s = self.clone();
        s.clock = Some(clock);
        s
    }
}

/// [ObsKey] is used to Index [Observations] in [Record] type definition.
#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ObsKey {
    /// Sampling [Epoch]
    pub epoch: Epoch,
    /// [EpochFlag] gives more information about sampling conditions
    pub flag: EpochFlag,
}

/// Observation [Record] are sorted by [Epoch] of observation and may have two different forms.
pub type Record = BTreeMap<ObsKey, Observations>;

// #[cfg(feature = "obs")]
// #[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
// #[derive(Debug, Copy, Clone)]
// pub enum Combination {
//     GeometryFree,
//     IonosphereFree,
//     WideLane,
//     NarrowLane,
//     MelbourneWubbena,
// }

// /// GNSS signal combination trait.
// /// This only applies to OBS RINEX records.
// /// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1]
// /// for more information.
// #[cfg(feature = "obs")]
// #[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
// pub trait Combine {
//     fn combine(
//         &self,
//         combination: Combination,
//     ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
// }

// /// GNSS code bias estimation trait.
// /// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1].
// #[cfg(feature = "obs")]
// #[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
// pub trait Dcb {
//     /// Returns Differential Code Bias estimates, sorted per (unique)
//     /// signals combinations and for each individual SV.
//     /// ```
//     /// use rinex::prelude::*;
//     /// use rinex::observation::*; // .dcb()
//     ///
//     /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
//     ///    .unwrap();
//     /// let dcb = rinex.dcb();
//     /// ```
//     fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
// }

// #[cfg(feature = "obs")]
// impl Combine for Record {
//     fn combine(
//         &self,
//         c: Combination,
//     ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
//         match c {
//             Combination::GeometryFree
//             | Combination::IonosphereFree
//             | Combination::NarrowLane
//             | Combination::WideLane => dual_freq_combination(self, c),
//             Combination::MelbourneWubbena => mw_combination(self),
//         }
//     }
// }
