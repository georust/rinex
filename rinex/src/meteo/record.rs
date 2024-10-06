use crate::{
    epoch, merge, merge::Merge, prelude::Duration, prelude::*, split, split::Split, types::Type,
    version, Observable,
};

use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

#[cfg(feature = "processing")]
use qc_traits::processing::{
    DecimationFilter, DecimationFilterType, FilterItem, MaskFilter, MaskOperand,
};

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] epoch::ParsingError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl Merge for Record {
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, observations) in rhs.iter() {
            if let Some(oobservations) = self.get_mut(epoch) {
                for (observation, data) in observations.iter() {
                    if !oobservations.contains_key(observation) {
                        // new observation
                        oobservations.insert(observation.clone(), *data);
                    }
                }
            } else {
                // new epoch
                self.insert(*epoch, observations.clone());
            }
        }
        Ok(())
    }
}

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k >= &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "processing")]
pub(crate) fn meteo_mask_mut(rec: &mut Record, mask: &MaskFilter) {
    match mask.operand {
        MaskOperand::Equals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e == *epoch),
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
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
                    rec.retain(|_, data| {
                        data.retain(|code, _| observables.contains(code));
                        !data.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::NotEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e != *epoch),
            FilterItem::ComplexItem(filter) => {
                // try to interprate as [Observable]
                let observables = filter
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
                    rec.retain(|_, data| {
                        data.retain(|code, _| !observables.contains(code));
                        !data.is_empty()
                    });
                }
            },
            _ => {},
        },
        MaskOperand::GreaterEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e >= *epoch),
            _ => {},
        },
        MaskOperand::GreaterThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e > *epoch),
            _ => {},
        },
        MaskOperand::LowerEquals => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e <= *epoch),
            _ => {},
        },
        MaskOperand::LowerThan => match &mask.item {
            FilterItem::EpochItem(epoch) => rec.retain(|e, _| *e < *epoch),
            _ => {},
        },
    }
}

#[cfg(feature = "processing")]
pub(crate) fn meteo_decim_mut(rec: &mut Record, f: &DecimationFilter) {
    if f.item.is_some() {
        todo!("targetted decimation not supported yet");
    }
    match f.filter {
        DecimationFilterType::Modulo(r) => {
            let mut i = 0;
            rec.retain(|_, _| {
                let retained = (i % r) == 0;
                i += 1;
                retained
            });
        },
        DecimationFilterType::Duration(interval) => {
            let mut last_retained = Option::<Epoch>::None;
            rec.retain(|e, _| {
                if let Some(last) = last_retained {
                    let dt = *e - last;
                    if dt >= interval {
                        last_retained = Some(*e);
                        true
                    } else {
                        false
                    }
                } else {
                    last_retained = Some(*e);
                    true // always retain 1st epoch
                }
            });
        },
    }
}
