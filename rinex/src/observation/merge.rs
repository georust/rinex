//! Merge implementation
use crate::{
    observation::Record,
    prelude::{Merge, MergeError},
};

impl Merge for Record {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        Ok(())
    }
}
