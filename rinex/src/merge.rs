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
pub fn merge_mut_unique_map2d<K: PartialEq + Eq + Hash + Clone, V: Clone> 
    (lhs: &mut HashMap<K, Vec<V>>, rhs: &HashMap<K, Vec<V>>) 
{
    for (k, values) in rhs.iter() {
        if !lhs.contains_key(&k) {
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
    fn merge_mut(&mut self, rhs: &T) -> Result<(), Error>;
}


#[derive(Clone, Debug)]
/// `RINEX` merging options
pub struct MergeOpts {
    /// optionnal program name
    pub program: String,
    /// timestamp where new file was appended
    pub date: chrono::NaiveDateTime, 
}

/*impl std::str::FromStr for MergeOpts {
    Err = MergeError;
    /// Builds MergeOpts structure from "standard" RINEX comment line
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (program, rem) = line.split_at(20);
        let (ops, rem) = rem.split_at(20);
        let (date, _) = rem.split_at(20);
        if !opts.trim().eq("FILE MERGE") {
            return Err(MergeError::MergeOptsDescriptionMismatch)
        }
        MergeOpts {
            program: program.trim().to_string(),
            date : chrono::DateTime::parse_from_str(date.split_at(16).0, "%Y%m%d %h%m%s")?, 
        }
    }
}*/
