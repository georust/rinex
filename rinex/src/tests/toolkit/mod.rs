use crate::navigation::FrameClass;
use crate::*;
use rand::{distributions::Alphanumeric, Rng};

// OBS RINEX dedicated tools
mod observation;
pub use observation::{
    generic_observation_epoch_decoding_test, generic_observation_rinex_against_model,
    generic_observation_rinex_test, ClockDataPoint, SignalDataPoint,
};

// NAV RINEX dedicated tools
pub mod nav;

// DORIS RINEX dedicated tools
mod doris;
pub use doris::check_observables as doris_check_observables;
pub use doris::check_stations as doris_check_stations;

// Meteo RINEX dedicated tests
mod meteo;
pub use meteo::{generic_meteo_rinex_against_model, generic_meteo_rinex_test};

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

/*
 * Navigation RINEX thorough comparison
 */
fn navigation_against_model(dut: &Rinex, model: &Rinex, filename: &str, _epsilon: f64) {
    let rec_dut = dut.record.as_nav().expect("failed to unwrap rinex record");
    let rec_model = model
        .record
        .as_nav()
        .expect("failed to unwrap rinex record");
    for (e_model, model_frames) in rec_model.iter() {
        if let Some(dut_frames) = rec_dut.get(e_model) {
            println!("{:?}", dut_frames);
            for model_frame in model_frames {
                let mut frametype = FrameClass::default();
                if model_frame.as_eph().is_some() {
                    frametype = FrameClass::Ephemeris;
                } else if model_frame.as_sto().is_some() {
                    frametype = FrameClass::SystemTimeOffset;
                } else if model_frame.as_eop().is_some() {
                    frametype = FrameClass::EarthOrientation;
                } else if model_frame.as_ion().is_some() {
                    frametype = FrameClass::IonosphericModel;
                }
                if !dut_frames.contains(model_frame) {
                    panic!(
                        "\"{}\" - @{} missing {} frame {:?}",
                        filename, e_model, frametype, model_frame
                    );
                    //assert_eq!(
                    //    observation_model, observation_dut,
                    //    "\"{}\" - {:?} - faulty \"{}\" observation - expecting {} - got {}",
                    //    filename, e_model, code_model, observation_model, observation_dut
                    //);
                }
            }
        } else {
            panic!("\"{}\" - missing epoch {:?}", filename, e_model);
        }
    }

    //for (e_dut, obscodes_dut) in rec_dut.iter() {
    //    if let Some(obscodes_model) = rec_model.get(e_dut) {
    //        for (code_dut, observation_dut) in obscodes_dut.iter() {
    //            if let Some(observation_model) = obscodes_model.get(code_dut) {
    //                assert_eq!(
    //                    observation_model, observation_dut,
    //                    "\"{}\" - {:?} - faulty \"{}\" observation - expecting {} - got {}",
    //                    filename, e_dut, code_dut, observation_model, observation_dut
    //                );
    //            } else {
    //                panic!(
    //                    "\"{}\" - {:?} parsed \"{}\" unexpectedly",
    //                    filename, e_dut, code_dut
    //                );
    //            }
    //        }
    //    } else {
    //        panic!("\"{}\" - parsed {:?} unexpectedly", filename, e_dut);
    //    }
    //}
}
