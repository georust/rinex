use crate::{
    observation::Record,
    prelude::{Duration, Epoch},
    split::{Error, Split},
};

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch < epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k.epoch >= epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, SplitError> {
        let mut curr = Self::new();
        let mut ret: Vec<Self> = Vec::new();
        let mut prev: Option<Epoch> = None;
        for (key, data) in self {
            if let Some(p_epoch) = prev {
                let dt = *epoch - p_epoch;
                if dt >= duration {
                    prev = Some(*epoch);
                    ret.push(curr);
                    curr = Self::new();
                }
                curr.insert((*epoch, *flag), data.clone());
            } else {
                prev = Some(*epoch);
            }
        }
        Ok(ret)
    }
}
