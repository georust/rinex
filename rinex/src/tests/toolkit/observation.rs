// use crate::observation::Record as ObsRecord;
use crate::prelude::{Constellation, Epoch, Observable, Rinex};
use crate::tests::toolkit::{
    test_gnss_csv, test_observables_csv, test_rinex, test_sv_csv, test_time_frame, TestTimeFrame,
};
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
                    "no observable in header for {} constellation",
                    constellation
                );
            }
        },
        _ => {
            panic!("empty observation specific header fields");
        },
    }
}

/*
 * Any parsed OBSERVATION RINEX should go through this test
 */
pub fn test_observation_rinex(
    dut: &Rinex,
    version: &str,
    constellation: Option<&str>,
    gnss_csv: &str,
    sv_csv: &str,
    observ_csv: &str,
    time_of_first_obs: Option<&str>,
    time_of_last_obs: Option<&str>,
    time_frame: TestTimeFrame,
    //observ_gnss_json: &str,
) {
    test_rinex(dut, version, constellation);
    assert!(
        dut.is_observation_rinex(),
        "should be declared as OBS RINEX"
    );

    assert!(
        dut.record.as_obs().is_some(),
        "observation record unwrapping"
    );
    test_sv_csv(dut, sv_csv);
    test_gnss_csv(dut, gnss_csv);
    test_time_frame(dut, time_frame);
    test_observables_csv(dut, observ_csv);
    /*
     * Specific header field testing
     */
    assert!(
        dut.header.obs.is_some(),
        "missing observation specific header fields"
    );
    assert!(
        dut.header.meteo.is_none(),
        "should not contain specific METEO fields"
    );
    assert!(
        dut.header.ionex.is_none(),
        "should not contain specific IONEX fields"
    );
    assert!(
        dut.header.clock.is_none(),
        "should not contain specific CLOCK fields"
    );

    let header = dut.header.obs.as_ref().unwrap();
    //for (constell, observables) in observables {
    //    assert!(header_obs.codes.get(&constell).is_some(), "observation rinex specific header missing observables for constellation {}", constell);
    //    let values = header_obs.codes.get(&constell).unwrap();
    //    for o in &observables {
    //        assert!(values.contains(&o), "observation rinex specific {} header is missing {} observable", constell, o);
    //    }
    //    for o in values {
    //        assert!(values.contains(&o), "observation rinex specific {} header should not contain {} observable", constell, o);
    //    }
    //}
    if let Some(time_of_first_obs) = time_of_first_obs {
        assert_eq!(
            Some(Epoch::from_str(time_of_first_obs).unwrap()),
            header.time_of_first_obs,
            "obs header is missing time of first obs \"{}\"",
            time_of_first_obs
        );
    }
    if let Some(time_of_last_obs) = time_of_last_obs {
        assert_eq!(
            Some(Epoch::from_str(time_of_last_obs).unwrap()),
            header.time_of_last_obs,
            "obs header is missing time of last obs \"{}\"",
            time_of_last_obs
        );
    }
}
