//! Observation RINEX module
use std::collections::HashMap;

use crate::{
    prelude::{Epoch, SV},
    version::Version,
    observable::Observable,
};

mod snr;
mod crinex;
mod observation;

pub mod flag;
pub mod record;
pub mod header;

pub use snr::SNR;
pub use flag::EpochFlag;
pub use crinex::Crinex;
pub use header::HeaderFields;
pub use record::{LliFlags, ObsKey, Observation, Record};
pub use observation::{Observation, SignalObservation};

#[cfg(docsrs)]
use crate::Bibliography;

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

#[cfg(feature = "processing")]
use itertools::Itertools;


#[cfg(feature = "obs")]
use std::collections::BTreeMap;

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