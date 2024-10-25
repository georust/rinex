//! Split implementation
use crate::{
    observation::Record,
    prelude::{Duration, Epoch, Split},
};

impl Split for Record {
    fn split(&self, epoch: Epoch) -> (Self, Self) {
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
    fn split_mut(&mut self, t: Epoch) -> Self {
        Self::default()
    }
    fn split_even_dt(&self, _: Duration) -> Vec<Self> {
        Default::default()
    }
}
