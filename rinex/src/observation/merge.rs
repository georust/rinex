//! Merge implementation
use crate::{
    merge::{Error as MergeError, Merge},
    observation::Record,
};

impl Merge for Record {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    fn merge_mut(&mut self, _rhs: &Self) -> Result<(), MergeError> {
        Ok(())
    }
}
