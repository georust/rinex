//! Observation Record specific header fields

use crate::{
    fmt_rinex,
    hatanaka::CRINEX,
    prelude::{Constellation, Epoch, Observable, TimeScale, FormattingError},
};

use std::{
    collections::HashMap, 
    io::{BufWriter, Write},
    str::FromStr,
};

#[cfg(feature = "processing")]
use itertools::Itertools;

#[cfg(feature = "processing")]
use qc_traits::{FilterItem, MaskFilter, MaskOperand};

#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optional CRINEX information
    pub crinex: Option<CRINEX>,
    /// [Epoch] of first observation. Following content should match.
    /// Defines [TimeScale] of following content.
    pub timeof_first_obs: Option<Epoch>,
    /// [Epoch] of last observation. Following content should match.
    /// Defines [TimeScale] of following content.
    pub timeof_last_obs: Option<Epoch>,
    /// Observables per constellation basis
    pub codes: HashMap<Constellation, Vec<Observable>>,
    /// True if local clock drift is compensated for
    pub clock_offset_applied: bool,
    /// Possible observation scaling, used in high precision
    /// OBS RINEX (down to nano radians precision).
    pub scaling: HashMap<(Constellation, Observable), u16>,
}

impl HeaderFields {

    /// Formats [HeaderFields] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>, major: u8) -> Result<(), FormattingError> {
        match major {
            1 | 2 => {
                if let Some((_constell, observables)) = self.codes.iter().next() {
                    write!(w, "{:6}", observables.len())?;
                    for (nth, observable) in observables.iter().enumerate() {
                        if (nth % 9) == 8 {
                            write!(w, "# / TYPES OF OBSERV\n      ")?;
                        }
                        write!(w, "    {}", observable)?;
                    }
                }
            },
            _ => {
                for (constell, observables) in &self.codes {
                    write!(
                        w,
                        "{:x}{:5}",
                        constell,
                        observables.len(),
                    )?;
                    for (nth, observable) in observables.iter().enumerate() {
                        if (nth % 13) == 12 {
                            write!(w, "SYS / # / OBS TYPES\n        ")?;
                        }
                        write!(w, " {}", observable)?;
                    }
                }
            },
        }

        // must take place after list of observables:
        //  TODO DCBS compensations
        //  TODO PCVs compensations

        Ok(())
    }

    /// Add "TIME OF FIRST OBS" field
    pub(crate) fn with_timeof_first_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.timeof_first_obs = Some(epoch);
        s
    }

    /// Add "TIME OF LAST OBS" field
    pub(crate) fn with_timeof_last_obs(&self, epoch: Epoch) -> Self {
        let mut s = self.clone();
        s.timeof_last_obs = Some(epoch);
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

impl HeaderFields {
    /// Timescale helper
    pub(crate) fn timescale(&self) -> TimeScale {
        match self.timeof_first_obs {
            Some(ts) => ts.time_scale,
            None => match self.timeof_last_obs {
                Some(ts) => ts.time_scale,
                None => TimeScale::GPST,
            },
        }
    }
}

#[cfg(feature = "processing")]
impl HeaderFields {
    /// Modifies in place Self, when applying preprocessing filter ops
    pub(crate) fn mask_mut(&mut self, f: &MaskFilter) {
        match f.operand {
            MaskOperand::Equals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    self.timeof_last_obs = Some(epoch.to_time_scale(ts));
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
                    if let Some(t) = self.timeof_first_obs {
                        if t < *epoch {
                            self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_first) = self.timeof_first_obs {
                        if t_first < *epoch {
                            self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_first_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
            MaskOperand::LowerThan => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.timeof_last_obs {
                        if t_last > *epoch {
                            self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_last_obs = Some(*epoch);
                    }
                },
                _ => {},
            },
            MaskOperand::LowerEquals => match &f.item {
                FilterItem::EpochItem(epoch) => {
                    let ts = self.timescale();
                    if let Some(t_last) = self.timeof_last_obs {
                        if t_last > *epoch {
                            self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                        }
                    } else {
                        self.timeof_last_obs = Some(epoch.to_time_scale(ts));
                    }
                },
                _ => {},
            },
        }
    }
}
