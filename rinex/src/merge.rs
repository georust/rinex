//! RINEX File merging (combination)
use crate::prelude::Epoch;
use hifitime::EpochError;
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::hash::Hash;
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

pub trait Merge {
    /// Merge "rhs" dataset into self, to form a new dataset.
    /// When merging two RINEX toghether, the data records
    /// remain sorted by epoch in chrnonological order.
    /// The merge operation behavior differs when dealing with
    /// either a/the header sections, than dealing with the record set.
    /// When dealing with the header sections, the behavior is to
    /// preserve existing attributes and only new information contained
    /// in "rhs" is introduced.  
    /// When dealing with the record set, "lhs" content is completely
    /// overwritten, that means:
    ///   - existing epochs get replaced
    ///   - new epochs can be introduced
    /// This currently is the only behavior supported.
    /// It was mainly developped for two reasons:
    ///   - allow meaningful and high level operations
    /// in a data production context
    ///   - allow complex operations on a data subset,
    /// where only specific data subset fields are targetted by
    /// said operation, in preprocessing toolkit.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::merge::Merge;
    /// let rnx_a = Rinex::from_file("../test_resources/OBS/V2/delf0010.21o")
    ///     .unwrap();
    /// let rnx_b = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
    ///     .unwrap();
    /// let merged = rnx_a.merge(&rnx_b);
    /// // When merging, RINEX format must match
    /// assert_eq!(merged.is_ok(), false);
    /// let rnx_b = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///     .unwrap();
    /// let merged = rnx_a.merge(&rnx_b);
    /// assert_eq!(merged.is_ok(), true);
    /// let merged = merged.unwrap();
    /// // when merging, Self's attributes are always prefered.
    /// // Results have most delf0010.21o attributes
    /// // Only new attributes, that 'DUTH0630.22O' would introduced are stored
    /// assert_eq!(merged.header.version.major, 2);
    /// assert_eq!(merged.header.version.minor, 11);
    /// assert_eq!(merged.header.program, "teqc  2019Feb25");
    /// // Resulting RINEX will therefore follow RINEX2 specifications
    /// assert!(merged.to_file("merge.rnx").is_ok(), "failed to merge file");
    /// ```
    fn merge(&self, rhs: &Self) -> Result<Self, Error>
    where
        Self: Sized;

    /// [Self::merge] mutable implementation.
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), Error>;
}
