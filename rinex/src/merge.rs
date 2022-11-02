//! `merging` operations related definitions 
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
    fn merge(&self, rhs: &T) -> Result<Self, Error> where Self: Sized;
    fn merge_mut(&mut self, rhs: &T) -> Result<(), Error>;
}
