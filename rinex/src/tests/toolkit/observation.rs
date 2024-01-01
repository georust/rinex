// use crate::observation::Record as ObsRecord;
use crate::prelude::{Constellation, Observable, Rinex};
use std::str::FromStr;

/*
 * Verifies given constellation does (only) contain the following observables
 */
pub fn check_observables(rnx: &Rinex, constellation: Constellation, observables: &[&str]) {
    let expected = observables
        .iter()
        .map(|desc| Observable::from_str(desc).unwrap())
        .collect::<Vec<_>>();

    match &rnx.header.obs {
        Some(obs_specific) => {
            let observables = obs_specific.codes.get(&constellation);
            if let Some(observables) = observables {
                for expected in &expected {
                    let mut found = false;
                    for observable in observables {
                        found |= observable == expected;
                    }
                    if !found {
                        panic!(
                            "{} observable is not present in header, for {} constellation",
                            expected, constellation
                        );
                    }
                }
                for observable in observables {
                    let mut is_expected = false;
                    for expected in &expected {
                        is_expected |= expected == observable;
                    }
                    if !is_expected {
                        panic!(
                            "{} header observables unexpectedly contain {} observable",
                            constellation, observable
                        );
                    }
                }
            } else {
                panic!(
                    "no observable in header, for {} constellation",
                    constellation
                );
            }
        },
        _ => {
            panic!("empty observation specific header fields");
        },
    }
}
