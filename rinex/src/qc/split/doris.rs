use crate::{
    doris::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, epoch: Epoch) -> (Record, Record) {
    let r0 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.0 <= epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    let r1 = rec
        .iter()
        .flat_map(|(k, v)| {
            if k.0 > epoch {
                Some((*k, v.clone()))
            } else {
                None
            }
        })
        .collect();

    (r0, r1)
}

pub fn split_mut(rec: &mut Record, t: Epoch) -> Record {
    let r1 = rec
        .iter()
        .flat_map(|(k, v)| if k.0 > t { Some((*k, v.clone())) } else { None })
        .collect();

    rec.retain(|k, _| k.0 < t);
    r1
}

pub fn split_even_dt(rec: &Record, _duration: Duration) -> Vec<Record> {
    Vec::new()
}
