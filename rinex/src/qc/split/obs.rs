use crate::{
    observation::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, epoch: Epoch) -> (Record, Record) {
    let r0 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch < epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    (r0, r1)
}

pub fn split_mut(rec: &mut Record, epoch: Epoch) -> Record {
    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    rec.retain(|k, _| k.epoch < epoch);
    r1
}

pub fn split_even_dt(rec: &Record, _duration: Duration) -> Vec<Record> {
    Vec::new()
}
