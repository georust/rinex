use crate::*;
use rand::{distributions::Alphanumeric, Rng};

// OBS RINEX dedicated tools
mod observation;
pub use observation::{
    generic_comparison as generic_observation_comparison, generic_observation_epoch_decoding_test,
    generic_observation_rinex_test, SignalDataPoint,
};

// IONEX test toolkit
mod ionex;
pub use ionex::{generic_ionex_test, TecPoint};

// NAV RINEX dedicated tools
mod nav;
pub use nav::{
    generic_comparison as generic_navigation_comparison, generic_test as generic_navigation_test,
};

// DORIS RINEX dedicated tools
mod doris;
pub use doris::check_observables as doris_check_observables;
pub use doris::check_stations as doris_check_stations;

// Meteo RINEX dedicated tests
mod meteo;
pub use meteo::{generic_comparison as generic_meteo_comparison, generic_meteo_rinex_test};

pub mod timeframe;
pub use timeframe::TimeFrame;

pub mod csv;
pub use csv::{gnss_csv, observables_csv, sv_csv};

/// Random name generator
pub fn random_name(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/// Generic sampling test
pub fn generic_timeframe_test(dut: &Rinex, tf: TimeFrame) {
    // grab epoch iter
    let mut dut = dut.epoch_iter();
    let mut model = tf.into_iter();

    while let Some(model) = model.next() {
        if let Some(dut) = dut.next() {
            assert_eq!(model, dut);
        } else {
            panic!("missing temporal data {:?}", model);
        }
    }

    let next = dut.next();
    assert!(
        next.is_none(),
        "timeframe exceeded! unexpected content {:?}",
        next.unwrap()
    );
}

/// Generic test that may apply to any [Rinex].
/// Verifies
///   - [Version]
///   - [Constellation]
///   - [RinexType]
///   - Possible [TimeFrame] for complex sampling testing
pub fn generic_rinex_test(
    dut: &Rinex,
    version: &str,
    constellation: Option<&str>,
    expected_type: RinexType,
    timeframe: Option<TimeFrame>,
) {
    assert_eq!(dut.header.rinex_type, expected_type);

    let version = Version::from_str(version).unwrap();
    assert_eq!(dut.header.version, version);

    let constellation = constellation.map(|s| Constellation::from_str(s.trim()).unwrap());
    assert_eq!(dut.header.constellation, constellation);

    if let Some(tf) = timeframe {
        generic_timeframe_test(dut, tf);
    }
}

/// Verifies that all contained data is Constant with Epsilon tolerance
pub fn generic_constant_rinex_test(dut: &Rinex, constant: f64, epsilon: f64) {
    if let Some(rec) = dut.record.as_obs() {
        for (k, v) in rec.iter() {
            for signal in v.signals.iter() {
                let err = (signal.value - constant).abs();
                assert!(
                    err < epsilon,
                    "({}{}) {} != {}",
                    k.epoch,
                    signal.sv,
                    signal.observable,
                    constant
                );
            }
        }
    }
}

/// Verifies that all contained data is Null
pub fn generic_null_rinex_test(dut: &Rinex) {
    generic_constant_rinex_test(dut, 0.0, 1.0E-9);
}

/// Compares strict equality between [A, B]
/// for all supported types, with panic on any single error;
/// and meaningful error report
pub fn generic_rinex_comparison(dut: &Rinex, model: &Rinex) {
    // Headers strict equality
    assert_eq!(
        dut.header, model.header,
        "header mismatch, got {:#?} expecting {:#?}",
        dut.header, model.header
    );

    if dut.is_observation_rinex() && model.is_observation_rinex() {
        generic_observation_comparison(&dut, &model);
    } else if dut.is_meteo_rinex() && model.is_meteo_rinex() {
        generic_meteo_comparison(&dut, &model);
    } else if dut.is_navigation_rinex() && model.is_navigation_rinex() {
        generic_navigation_comparison(&dut, &model);
    }
}

/*
 * CLOCK Rinex thorough comparison
 */
fn clocks_against_model(dut: &Rinex, model: &Rinex, filename: &str, _epsilon: f64) {
    let rec_dut = dut
        .record
        .as_clock()
        .expect("failed to unwrap rinex record");
    let rec_model = model
        .record
        .as_clock()
        .expect("failed to unwrap rinex record");
    for (e_model, model_types) in rec_model.iter() {
        if let Some(dut_types) = rec_dut.get(e_model) {
            for (model_data, _model_systems) in model_types.iter() {
                if let Some(_systems) = dut_types.get(model_data) {
                } else {
                    panic!(
                        "\"{}\" - {:?} - missing data {:?}",
                        filename, e_model, model_data
                    );
                }
            }
        } else {
            panic!("\"{}\" - missing epoch {:?}", filename, e_model);
        }
    }
}
