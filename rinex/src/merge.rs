//! [Merge] implementation

use crate::{
    prelude::{Epoch, Merge},
};

use hifitime::EpochError;

use std::{
    cmp::{Eq, PartialEq},
    collections::HashMap,
    hash::Hash,
};

use thiserror::Error;

/// Merge operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("file type mismatch: cannot merge different RINEX together")]
    FileTypeMismatch,
    #[error("cannot merge mixed absolute/relative phase antenna together")]
    AntexAbsoluteRelativeMismatch,
    #[error("cannot merge ionex based off different reference systems")]
    IonexReferenceMismatch,
    #[error("cannot merge ionex with different grid definition")]
    IonexMapGridMismatch,
    #[error("cannot merge ionex of different dimensions")]
    IonexMapDimensionsMismatch,
    #[error("cannot merge ionex where base radius differs")]
    IonexBaseRadiusMismatch,
    #[error("failed to retrieve system time for merge ops date")]
    HifitimeError(#[from] EpochError),
}

/*
 * Appends given vector into self.
 */
pub(crate) fn merge_mut_vec<T: Clone>(lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        lhs.push(item.clone());
    }
}

/*
 * Merges given vector into self, but ensures values are unique.
 */
pub(crate) fn merge_mut_unique_vec<T: Clone + PartialEq>(lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        if !lhs.contains(item) {
            lhs.push(item.clone());
        }
    }
}

/*
 * Merges given map into self but ensures both keys and values are unique.
 */
pub(crate) fn merge_mut_unique_map2d<K: PartialEq + Eq + Hash + Clone, V: Clone + PartialEq>(
    lhs: &mut HashMap<K, Vec<V>>,
    rhs: &HashMap<K, Vec<V>>,
) {
    for (k, values) in rhs.iter() {
        if let Some(vvalues) = lhs.get_mut(k) {
            for value in values {
                if !vvalues.contains(value) {
                    vvalues.push(value.clone());
                }
            }
        } else {
            lhs.insert(k.clone(), values.clone());
        }
    }
}

/*
 * Merges optionnal data fields, rhs overwrites lhs, only if lhs is not previously defined.
 */
pub(crate) fn merge_mut_option<T: Clone>(lhs: &mut Option<T>, rhs: &Option<T>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(rhs.clone());
        }
    }
}

/*
 * Merges "TIME OF FIRST" special OBSERVATION header field
 */
pub(crate) fn merge_time_of_first_obs(lhs: &mut Option<Epoch>, rhs: &Option<Epoch>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(*rhs);
        }
    } else if let Some(rhs) = rhs {
        let tl = lhs.unwrap();
        *lhs = Some(std::cmp::min(tl, *rhs));
    }
}

/*
 * Merges "TIME OF LAST" special OBSERVATION header field
 */
pub(crate) fn merge_time_of_last_obs(lhs: &mut Option<Epoch>, rhs: &Option<Epoch>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(*rhs);
        }
    } else if let Some(rhs) = rhs {
        let tl = lhs.unwrap();
        *lhs = Some(std::cmp::max(tl, *rhs));
    }
}


impl Merge for RINEX {
    /// Merges `rhs` into `Self` without mutable access, at the expense of memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self` in place
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        self.header.merge_mut(&rhs.header)?;
        if !self.is_antex() {
            if self.epoch().count() == 0 {
                // lhs is empty : overwrite
                self.record = rhs.record.clone();
            } else if rhs.epoch().count() != 0 {
                // real merge
                self.record.merge_mut(&rhs.record)?;
            }
        } else {
            // real merge
            self.record.merge_mut(&rhs.record)?;
        }
        Ok(())
    }
}