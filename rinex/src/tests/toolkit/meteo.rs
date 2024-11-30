use itertools::Itertools;
use std::str::FromStr;

use crate::{
    prelude::{Header, Observable, Rinex, RinexType},
    tests::toolkit::{generic_rinex_test, TimeFrame},
};

/// [Rinex] against [Rinex] model verification
pub fn generic_meteo_rinex_against_model(
    dut: &Rinex,
    model: &Rinex,
    filename: &str,
    _epsilon: f64,
) {
    let rec_dut = dut
        .record
        .as_meteo()
        .expect("failed to unwrap rinex record");

    let rec_model = model
        .record
        .as_meteo()
        .expect("failed to unwrap rinex record");

    // verify constellations
    let dut_sensors = dut.meteo_sensors_iter().collect::<Vec<_>>();
    let expected_sensors = model.meteo_sensors_iter().collect::<Vec<_>>();
    assert_eq!(dut_sensors, expected_sensors);

    // verify observables
    let dut_content = dut.observables_iter().sorted().collect::<Vec<_>>();
    let expected_content = dut.observables_iter().sorted().collect::<Vec<_>>();
    assert_eq!(dut_content, expected_content);

    for (k, _) in rec_dut.iter() {
        assert!(
            rec_model.get(k).is_some(),
            "found unexpected content: {:?}",
            k
        );
    }

    for (k, v) in rec_model.iter() {
        let dut = rec_dut.get(k).expect(&format!("missing content {:?}", k));
        assert_eq!(
            v, dut,
            "found invalid measurement {}({})",
            k.epoch, k.observable
        );
    }
}

/// Basic tests for Observation [Rinex]
fn basic_header_tests(dut: &Header) {
    assert!(dut.obs.is_none(),);
    assert!(dut.meteo.is_some(),);
    assert!(dut.ionex.is_none(),);
    assert!(dut.clock.is_none(),);

    let _ = dut.meteo.as_ref().expect("missing specific specs");
}

/// Generic test that we can use for Observation [Rinex]
pub fn generic_meteo_rinex_test(
    dut: &Rinex,
    model: Option<&Rinex>,
    version: &str,
    observables_csv: &str,
    time_frame: TimeFrame,
) {
    assert!(dut.is_meteo_rinex());

    let rec = dut.record.as_meteo().expect("meteo unwrapping failure");

    generic_rinex_test(
        dut,
        version,
        None,
        RinexType::MeteoData,
        //Some(time_frame), //TODO
        None,
    );

    basic_header_tests(&dut.header);

    // verify observables
    let model = observables_csv
        .split(',')
        .map(|ob| Observable::from_str(ob.trim()).unwrap())
        .sorted()
        .collect::<Vec<_>>();

    let observables = dut.observables_iter().cloned().collect::<Vec<_>>();

    assert_eq!(
        observables, model,
        "invalid observables {:?}/{:?}",
        observables, model
    );
}
