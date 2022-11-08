//! RINEX Merge operation
use thiserror::Error;
use std::collections::HashMap;
use std::cmp::{PartialEq, Eq};
use std::hash::Hash;

/// Merge operation related error(s)
#[derive(Error, Debug)]
pub enum Error {
    #[error("file type mismatch: cannot merge different types together")]
    FileTypeMismatch,
    #[error("cannot merge mixed absolute/relative phase antenna together")]
    AntexAbsoluteRelativeMismatch,
    #[error("cannot merge ionosphere maps based off different models")]
    IonexSystemMismatch,
    #[error("cannot merge ionosphere maps based off different grid system")]
    IonexMapGridMismatch,
    #[error("cannot merge ionosphere maps with different map dimensions")]
    IonexMapDimensionsMismatch,
    #[error("cannot merge ionosphere maps where base radius differs")]
    IonexBaseRadiusMismatch,
}

/// Appends given vector into self
pub fn merge_mut_vec<T: Clone> (lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        lhs.push(item.clone());
    }
}

/// Merges given vector into self, but ensures values are unique
pub fn merge_mut_unique_vec<T: Clone + PartialEq> (lhs: &mut Vec<T>, rhs: &Vec<T>) {
    for item in rhs {
        if !lhs.contains(&item) {
            lhs.push(item.clone());
        }
    }
}

/// Merges given map into self but ensures both keys and values are unique 
pub fn merge_mut_unique_map2d<K: PartialEq + Eq + Hash + Clone, V: Clone + PartialEq> 
    (lhs: &mut HashMap<K, Vec<V>>, rhs: &HashMap<K, Vec<V>>) 
{
    for (k, values) in rhs.iter() {
        if let Some(vvalues) = lhs.get_mut(&k) {
            for value in values {
                if !vvalues.contains(&value) {
                    vvalues.push(value.clone());
                }
            }
        } else {
            lhs.insert(k.clone(), values.clone());
        }
    }
}

/// Merges optionnal data field,
/// rhs overwrites lhs, only if lhs is not previously defined
pub fn merge_mut_option<T: Clone> (lhs: &mut Option<T>, rhs: &Option<T>) {
    if lhs.is_none() {
        if let Some(rhs) = rhs {
            *lhs = Some(rhs.clone());
        }
    }
}

pub trait Merge<T> {
    /// Merge immutable implementation.
    /// When merging, Self attributes are always prefered. 
    /// Only `rhs` new header information is introduced,
    /// rhs.header does not overwrite the previously known header attributes.
    /// Record is then form by combining both file bodies.
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
    /// merged.to_file("merge.rnx")
    ///     .unwrap();
    /// ```
    fn merge(&self, rhs: &T) -> Result<Self, Error> where Self: Sized;

    /// Merges Self and `rhs` into a single RINEX.
    /// See [merge] for an example of use.
    fn merge_mut(&mut self, rhs: &T) -> Result<(), Error>;
}
