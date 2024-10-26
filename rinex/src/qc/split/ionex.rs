use crate::{
    ionex::record::Record,
    prelude::{Duration, Epoch},
};

pub fn split(rec: &Record, t: Epoch) -> (Record, Record) {
    let before = rec
        .iter()
        .flat_map(|((e, h), plane)| {
            if *e < t {
                Some(((*e, *h), plane.clone()))
            } else {
                None
            }
        })
        .collect();

    let after = rec
        .iter()
        .flat_map(|((e, h), plane)| {
            if *e >= t {
                Some(((*e, *h), plane.clone()))
            } else {
                None
            }
        })
        .collect();

    (before, after)
}

pub fn split_mut(rec: &mut Record, epoch: Epoch) -> Record {
    let after = rec
        .iter()
        .flat_map(|((e, h), plane)| {
            if *e >= epoch {
                Some(((*e, *h), plane.clone()))
            } else {
                None
            }
        })
        .collect();

    rec.retain(|(t, _), _| *t < epoch);
    after
}

pub fn split_even_dt(rec: &Record, _duration: Duration) -> Vec<Record> {
    Vec::new()
}
