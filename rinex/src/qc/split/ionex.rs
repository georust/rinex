use crate::{
    ionex::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, t: Epoch) -> (Record, Record) {
    let before = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch < t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    let after = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    (before, after)
}

pub fn split_mut(rec: &mut Record, t: Epoch) -> Record {
    let after = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.epoch >= t {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    rec.retain(|k, _| k.epoch < t);

    after
}

pub fn split_even_dt(rec: &Record, _duration: Duration) -> Vec<Record> {
    Vec::new()
}
