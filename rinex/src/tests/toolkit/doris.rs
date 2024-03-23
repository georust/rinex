// use crate::observation::Record as ObsRecord;
use crate::prelude::{Observable, Rinex};
use std::str::FromStr;

/*
 * Verifies observable list
 */
pub fn check_observables(rnx: &Rinex, observables: &[&str]) {
    let expected = observables
        .iter()
        .map(|desc| Observable::from_str(desc).unwrap())
        .collect::<Vec<_>>();

    match &rnx.header.doris {
        Some(specific) => {
            if specific.observables.is_empty() {
                panic!("no observable in header seems suspicious");
            }
            for expected in &expected {
                if !specific.observables.contains(&expected) {
                    panic!("{} observable is not present in header", expected,);
                }
            }
            for identified in &specific.observables {
                if !expected.contains(&identified) {
                    panic!("identified unexpected observable: {}", identified);
                }
            }
        },
        _ => {
            panic!("missing header specific fields");
        },
    }
}
