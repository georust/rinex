use crate::{doris::Record, prelude::qc::MergeError};

pub fn merge_mut(rec: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    panic!("merge doris");
}
