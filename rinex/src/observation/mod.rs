//! Observation RINEX module
use super::{
    hatanaka::CRINEX,
    prelude::{Constellation, Epoch, Observable, SV},
    version::Version,
};

use std::collections::HashMap;

pub mod record;

pub mod flag;
pub use flag::EpochFlag;

mod snr;
pub use snr::SNR;

#[cfg(docsrs)]
use crate::Bibliography;

pub use record::{LliFlags, ObservationData, Record};

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

#[cfg(feature = "processing")]
use itertools::Itertools;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

use serde::Serialize;

/// Observation Record specific header fields
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information
    pub crinex: Option<CRINEX>,
    /// Time of FIRST OBS
    pub time_of_first_obs: Option<Epoch>,
    /// Time of LAST OBS
    pub time_of_last_obs: Option<Epoch>,
    /// Observables per constellation basis
    pub codes: HashMap<Constellation, Vec<Observable>>,
    /// True if local clock drift is compensated for
    pub clock_offset_applied: bool,
    /// Possible observation scaling, used in high precision
    /// OBS RINEX (down to nano radians precision).
    pub scaling: HashMap<(Constellation, Observable), u16>,
}

impl HeaderFields {
    /// Add TIME OF FIRST OBS
    pub(crate) fn with_time_of_first_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_first_obs = Some(epoch);
        s
    }
    /// Add TIME OF LAST OBS
    pub(crate) fn with_time_of_last_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.time_of_last_obs = Some(epoch);
        s
    }
    /// Insert a data scaling
    pub(crate) fn with_scaling(&mut self, c: Constellation, observable: Observable, scaling: u16) {
        self.scaling.insert((c, observable.clone()), scaling);
    }
    /// Returns given scaling to apply for given GNSS system
    /// and given observation. Returns 1.0 by default, so it always applies
    pub(crate) fn scaling(&self, c: Constellation, observable: Observable) -> Option<&u16> {
        self.scaling.get(&(c, observable))
    }
}

#[cfg(feature = "processing")]
use std::str::FromStr;

#[cfg(feature = "processing")]
impl HeaderFields {
    /// Timescale helper
    fn timescale(&self) -> TimeScale {
        match self.time_of_first_obs {
            Some(ts) => ts.time_scale,
            None => match self.time_of_last_obs {
                Some(ts) => ts.time_scale,
                None => TimeScale::GPST,
            },
        }
    }
    /// Modifies in place Self, when applying preprocessing filter ops
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    self.time_of_first_obs = Some(epoch.to_time_scale(ts));
                    self.time_of_last_obs = Some(epoch.to_time_scale(ts));
                },
                FilterItem::SvItem(svs) => {
                    let constells = svs
                        .iter()
                        .map(|sv| sv.constellation)
                        .unique()
                        .collect::<Vec<_>>();
                    self.codes.retain(|c, _| constells.contains(&c));
                    self.scaling.retain(|(c, _), _| constells.contains(&c));
                },
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if observables.len() > 0 {
                        self.codes.retain(|_, obs| {
                            obs.retain(|ob| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        self.scaling.retain(|(_, c), _| !observables.contains(c));
                    }
                },
                FilterItem::ConstellationItem(constells) => {
                    self.codes.retain(|c, _| constells.contains(&c));
                    self.scaling.retain(|(c, _), _| constells.contains(&c));
                },
                _ => {},
            },
            MaskOperand::NotEquals => match &f.item {
                FilterItem::SvItem(svs) => {
                    let constells = svs
                        .iter()
                        .map(|sv| sv.constellation)
                        .unique()
                        .collect::<Vec<_>>();
                    self.codes.retain(|c, _| !constells.contains(&c));
                    self.scaling.retain(|(c, _), _| !constells.contains(&c));
                },
                FilterItem::ConstellationItem(constells) => {
                    self.codes.retain(|c, _| !constells.contains(&c));
                    self.scaling.retain(|(c, _), _| !constells.contains(&c));
                },
                FilterItem::ComplexItem(complex) => {
                    // try to interprate as [Observable]
                    let observables = complex
                        .iter()
                        .filter_map(|f| {
                            if let Ok(ob) = Observable::from_str(f) {
                                Some(ob)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    if observables.len() > 0 {
                        self.codes.retain(|_, obs| {
                            obs.retain(|ob| observables.contains(&ob));
                            !obs.is_empty()
                        });
                        self.scaling.retain(|(_, c), _| !observables.contains(c));
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t) = self.time_of_first_obs {
                        if t < *epoch {
                            self.time_of_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.time_of_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_first) = self.time_of_first_obs {
                        if t_first < *epoch {
                            self.time_of_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.time_of_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.time_of_last_obs {
                        if t_last > *epoch {
                            self.time_of_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.time_of_last_obs = Some(*epoch);
                    }
                },
                _ => {},
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.time_of_last_obs {
                        if t_last > *epoch {
                            self.time_of_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.time_of_last_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
        }
    }
}

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
