//! Split implementation
use crate::{
    observation::Record,
    prelude::{Duration, Epoch},
    split::{Error as SplitError, Split},
};

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), SplitError> {
        let before = self
            .iter()
            .filter_map(|(key, sig)| {
                if key.epoch <= epoch {
                    Some((key.clone(), sig.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let after = self
            .iter()
            .filter_map(|(key, sig)| {
                if key.epoch > epoch {
                    Some((key.clone(), sig.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        Ok((Self::from_iter(before), Self::from_iter(after)))
    }
    fn split_dt(&self, _: Duration) -> Result<Vec<Self>, SplitError> {
        Err(SplitError::NoEpochIteration)
    }
}
