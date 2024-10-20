//! Observation RINEX module
use super::{epoch, prelude::*, version::Version};
use std::collections::HashMap;

pub use signal::SignalObservation;

mod clock;
mod crinex;
mod formatting;
mod header;
mod lli;
mod merge;
mod parsing;
mod signal;
mod snr;
mod split;

#[cfg(feature = "processing")]
mod mask;

#[cfg(feature = "processing")]
mod decim;

pub mod flag;
pub mod record;

pub use clock::ClockObservation;
pub use crinex::Crinex;
pub use flag::EpochFlag;
pub use lli::LliFlags;
pub use snr::SNR;

#[cfg(docsrs)]
use crate::Bibliography;

use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{epoch::ParsingError as EpochParsingError, observation::flag::Error as FlagError};

use gnss::{
    constellation::ParsingError as ConstellationParsingError, sv::ParsingError as SVParsingError,
};

/// Observation RINEX specific [ParsingError]
#[derive(Error, Debug)]
pub enum ParsingError {
    #[error("failed to parse epoch flag")]
    EpochFlag(#[from] FlagError),
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("sv parsing error")]
    SvParsing(#[from] SVParsingError),
    #[error("failed to parse vehicles properly (nb_sat mismatch)")]
    EpochParsingError,
    #[error("bad v2 satellites description")]
    BadV2SatellitesDescription,
    #[error("epoch is empty")]
    EmptyEpoch,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Observation {
    /// GNSS Receiver [ClockObservation]
    Clock(ClockObservation),
    /// GNSS [SignalObservation]
    Signal(SignalObservation),
}

impl Observation {
    /// Creates new [ClockObservation] with offset to [Constellation] in [s]
    pub fn clock_offset(offset_s: f64) -> Self {
        Self::Clock(ClockObservation { offset_s })
    }
    /// [ClockObservation] unwrapping attempt
    pub fn as_clock(&self) -> Option<&ClockObservation> {
        match self {
            Self::Clock(clock) => Some(clock),
            _ => None,
        }
    }
    /// Mutable [ClockObservation] unwrapping attempt    
    pub fn as_clock_mut(&mut self) -> Option<&mut ClockObservation> {
        match self {
            Self::Clock(clock) => Some(clock),
            _ => None,
        }
    }
    /// [SignalObservation] unwrapping attempt
    pub fn as_signal(&self) -> Option<&SignalObservation> {
        match self {
            Self::Signal(signal) => Some(signal),
            _ => None,
        }
    }
    /// Mutable [SignalObservation] unwrapping attempt
    pub fn as_signal_mut(&self) -> Option<&mut SignalObservation> {
        match self {
            Self::Signal(signal) => Some(signal),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ObsKey {
    /// Sampling [Epoch]
    pub epoch: Epoch,
    /// [EpochFlag] gives more information about sampling conditions
    pub flag: EpochFlag,
}

/// Observation [Record] are sorted by [Epoch] of observation and may have two different forms.
pub type Record = BTreeMap<ObsKey, Observation>;

#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
#[derive(Debug, Copy, Clone)]
pub enum Combination {
    GeometryFree,
    IonosphereFree,
    WideLane,
    NarrowLane,
    MelbourneWubbena,
}

/// GNSS signal combination trait.    
/// This only applies to OBS RINEX records.  
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1]
/// for more information.
#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
pub trait Combine {
    fn combine(
        &self,
        combination: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

/// GNSS code bias estimation trait.
/// Refer to [Bibliography::ESAGnssCombination] and [Bibliography::ESABookVol1].
#[cfg(feature = "obs")]
#[cfg_attr(docsrs, doc(cfg(feature = "obs")))]
pub trait Dcb {
    /// Returns Differential Code Bias estimates, sorted per (unique)
    /// signals combinations and for each individual SV.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::observation::*; // .dcb()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///    .unwrap();
    /// let dcb = rinex.dcb();
    /// ```
    fn dcb(&self) -> HashMap<String, BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>>;
}

#[cfg(feature = "obs")]
impl Combine for Record {
    fn combine(
        &self,
        c: Combination,
    ) -> HashMap<(Observable, Observable), BTreeMap<SV, BTreeMap<(Epoch, EpochFlag), f64>>> {
        match c {
            Combination::GeometryFree
            | Combination::IonosphereFree
            | Combination::NarrowLane
            | Combination::WideLane => dual_freq_combination(self, c),
            Combination::MelbourneWubbena => mw_combination(self),
        }
    }
}
