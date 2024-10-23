use crate::navigation::FrameClass;
use crate::*;
use rand::{distributions::Alphanumeric, Rng};

use hifitime::TimeSeries;

/* OBS RINEX dedicated tools */
mod observation;
pub use observation::check_observables as obsrinex_check_observables;
pub use observation::test_observation_rinex;

/* NAV RINEX dedicated tools */
pub mod nav;

/* DORIS RINEX dedicated tools */
mod doris;
pub use doris::check_observables as doris_check_observables;
pub use doris::check_stations as doris_check_stations;

/* ANY RINEX == constant (special ops) */
mod constant;
pub use constant::is_null_rinex;

//#[macro_use]
#[macro_export]
macro_rules! erratic_time_frame {
    ($csv: expr) => {
        TestTimeFrame::Erratic(
            $csv.split(",")
                .map(|c| Epoch::from_str(c.trim()).unwrap())
                .unique()
                .collect::<Vec<Epoch>>(),
        )
    };
}

#[macro_export]
macro_rules! evenly_spaced_time_frame {
    ($start: expr, $end: expr, $step: expr) => {
        TestTimeFrame::EvenlySpaced(TimeSeries::inclusive(
            Epoch::from_str($start.trim()).unwrap(),
            Epoch::from_str($end.trim()).unwrap(),
            Duration::from_str($step.trim()).unwrap(),
        ))
    };
}

#[derive(Debug, Clone)]
pub enum TestTimeFrame {
    Erratic(Vec<Epoch>),
    EvenlySpaced(TimeSeries),
}

impl TestTimeFrame {
    pub fn evenly_spaced(&self) -> Option<TimeSeries> {
        match self {
            Self::EvenlySpaced(ts) => Some(ts.clone()),
            _ => None,
        }
    }
    pub fn erratic(&self) -> Option<Vec<Epoch>> {
        match self {
            Self::Erratic(ts) => Some(ts.clone()),
            _ => None,
        }
    }
}

/*
 * Tool to generate random names when we need to produce a file
 */
pub fn random_name(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

/*
 * Creates list of observables
 */
pub fn build_observables(observable_csv: &str) -> Vec<Observable> {
    observable_csv
        .split(',')
        .map(|c| {
            let c = c.trim();
            if let Ok(observ) = Observable::from_str(c) {
                observ
            } else {
                panic!("invalid observable in csv");
            }
        })
        .collect::<Vec<Observable>>()
        .into_iter()
        .unique()
        .collect()
}

use std::str::FromStr;

/*
 * Build GNSS list
 */
pub fn build_gnss_csv(gnss_csv: &str) -> Vec<Constellation> {
    gnss_csv
        .split(',')
        .map(|c| Constellation::from_str(c.trim()).unwrap())
        .collect::<Vec<Constellation>>()
        .into_iter()
        .unique()
        .collect()
}

/*
 * Test method to compare one RINEX against GNSS content
 */
pub fn test_gnss_csv(dut: &Rinex, gnss_csv: &str) {
    let gnss = build_gnss_csv(gnss_csv);
    let dut_gnss: Vec<Constellation> = dut.constellation().collect();
    for g in &gnss {
        assert!(
            dut_gnss.contains(g),
            "dut does not contain constellation \"{}\"",
            g
        );
    }
    for g in &dut_gnss {
        assert!(
            gnss.contains(g),
            "dut should not contain constellation \"{:X}\"",
            g
        );
    }
}

/*
 * Compares one RINEX against SV total content
 */
pub fn test_sv_csv(dut: &Rinex, sv_csv: &str) {
    let sv: Vec<SV> = sv_csv
        .split(',')
        .map(|c| SV::from_str(c.trim()).unwrap())
        .collect::<Vec<SV>>()
        .into_iter()
        .unique()
        .collect();

    let dut_sv: Vec<SV> = dut.sv().collect();
    for v in &sv {
        assert!(dut_sv.contains(v), "dut does not contain vehicle \"{}\"", v);
    }
    for v in &sv {
        assert!(sv.contains(v), "dut should not contain vehicle \"{}\"", v);
    }
}

/*
 * Compares one RINEX against given epoch content
 */
pub fn test_time_frame(dut: &Rinex, tf: TestTimeFrame) {
    let mut dut_epochs = dut.epoch();
    let _epochs: Vec<Epoch> = Vec::new();
    if let Some(serie) = tf.evenly_spaced() {
        for e in serie {
            assert_eq!(
                Some(e),
                dut_epochs.next(),
                "dut does not contain epoch {}",
                e
            );
        }
        for e in dut_epochs.by_ref() {
            panic!("dut should not contain epoch {}", e);
        }
    } else if let Some(serie) = tf.erratic() {
        for e in serie {
            assert!(
                dut_epochs.any(|epoch| e == epoch),
                "dut does not contain epoch {}",
                e
            );
        }
        for e in dut_epochs {
            panic!("dut should not contain epoch {}", e);
        }
    }
}

/*
 * Tests provided vehicles per epoch
 * This is METEO + OBS compatible
 */
pub fn test_observables_csv(dut: &Rinex, observables_csv: &str) {
    let observ = build_observables(observables_csv);
    let dut_observ: Vec<&Observable> = dut.observable().collect();
    for o in &observ {
        assert!(
            dut_observ.contains(&o),
            "dut does not contain observable {}",
            o
        );
    }
    for o in &dut_observ {
        assert!(
            dut_observ.contains(o),
            "dut should not contain observable {}",
            o
        );
    }
}

/// Observation RINEX thorough comparison
fn observation_against_model(dut: &Rinex, model: &Rinex, filename: &str, epsilon: f64) {
    let rec_dut = dut.record.as_obs().expect("failed to unwrap rinex record");
    let rec_model = model
        .record
        .as_obs()
        .expect("failed to unwrap rinex record");

    // verify constellations
    let dut_constell: Vec<_> = dut.constellation().collect();
    let expected_constell: Vec<_> = model.constellation().collect();
    assert_eq!(
        dut_constell, expected_constell,
        "mismatch for \"{}\"",
        filename
    );

    for (key, value) in rec_model.iter() {
        let dut_value = rec_dut.get(key).expect(&format!("{} does not exist"), key);

        assert_eq!(value.clock, dut_value.clock);
        assert_eq!(value.signals, dut_value.signals);
    }
    for (key, _) in rec_dut.iter() {
        let model_value = rec_model
            .get(key)
            .expect(&format!("found unexpected key: {}", key));
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

/*
 * Meteo RINEX thorough comparison
 */
fn meteo_against_model(dut: &Rinex, model: &Rinex, filename: &str, _epsilon: f64) {
    let rec_dut = dut
        .record
        .as_meteo()
        .expect("failed to unwrap rinex record");
    let rec_model = model
        .record
        .as_meteo()
        .expect("failed to unwrap rinex record");
    for (e_model, obscodes_model) in rec_model.iter() {
        if let Some(obscodes_dut) = rec_dut.get(e_model) {
            for (code_model, observation_model) in obscodes_model.iter() {
                if let Some(observation_dut) = obscodes_dut.get(code_model) {
                    assert_eq!(
                        observation_model, observation_dut,
                        "\"{}\" - {:?} - faulty \"{}\" observation - expecting {} - got {}",
                        filename, e_model, code_model, observation_model, observation_dut
                    );
                } else {
                    panic!(
                        "\"{}\" - {:?} missing \"{}\" observation",
                        filename, e_model, code_model
                    );
                }
            }
        } else {
            panic!("\"{}\" - missing epoch {:?}", filename, e_model);
        }
    }

    for (e_dut, obscodes_dut) in rec_dut.iter() {
        if let Some(obscodes_model) = rec_model.get(e_dut) {
            for (code_dut, observation_dut) in obscodes_dut.iter() {
                if let Some(observation_model) = obscodes_model.get(code_dut) {
                    assert_eq!(
                        observation_model, observation_dut,
                        "\"{}\" - {:?} - faulty \"{}\" observation - expecting {} - got {}",
                        filename, e_dut, code_dut, observation_model, observation_dut
                    );
                } else {
                    panic!(
                        "\"{}\" - {:?} parsed \"{}\" unexpectedly",
                        filename, e_dut, code_dut
                    );
                }
            }
        } else {
            panic!("\"{}\" - parsed {:?} unexpectedly", filename, e_dut);
        }
    }
}

/*
 * Compares "dut" Device Under Test to given Model,
 * panics on unexpected content with detailed explanations.
 */
pub fn test_against_model(dut: &Rinex, model: &Rinex, filename: &str, epsilon: f64) {
    if dut.is_observation_rinex() {
        observation_against_model(dut, model, filename, epsilon);
    } else if dut.is_meteo_rinex() {
        meteo_against_model(dut, model, filename, epsilon);
    } else if dut.is_clock_rinex() {
        clocks_against_model(dut, model, filename, epsilon);
    } else if dut.is_navigation_rinex() {
        navigation_against_model(dut, model, filename, epsilon);
    }
}

/*
 * Any parsed RINEX should go through this test
 */
pub fn test_rinex(dut: &Rinex, version: &str, constellation: Option<&str>) {
    let version = Version::from_str(version).unwrap();
    assert!(
        dut.header.version == version,
        "parsed wrong version {}, expecting \"{}\"",
        dut.header.version,
        version
    );

    let constellation = constellation.map(|s| Constellation::from_str(s.trim()).unwrap());
    assert!(
        dut.header.constellation == constellation,
        "bad gnss description: {:?}, expecting {:?}",
        dut.header.constellation,
        constellation
    );
}

/*
 * Any parsed METEO RINEX should go through this test
 */
pub fn test_meteo_rinex(
    dut: &Rinex,
    version: &str,
    observables_csv: &str,
    time_frame: TestTimeFrame,
) {
    test_rinex(dut, version, None);
    assert!(dut.is_meteo_rinex(), "should be declared as METEO RINEX");
    test_observables_csv(dut, observables_csv);
    test_time_frame(dut, time_frame);
    /*
     * Header specific fields
     */
    assert!(
        dut.header.obs.is_none(),
        "should not contain specific OBS fields"
    );
    assert!(
        dut.header.meteo.is_some(),
        "should contain specific METEO fields"
    );
    assert!(
        dut.header.ionex.is_none(),
        "should not contain specific IONEX fields"
    );
    assert!(
        dut.header.clock.is_none(),
        "should not contain specific CLOCK fields"
    );

    let _header = dut.header.meteo.as_ref().unwrap();
}

/*
 * Any parsed NAVIGATION RINEX should go through this test
 */
pub fn test_navigation_rinex(dut: &Rinex, version: &str, constellation: Option<&str>) {
    test_rinex(dut, version, constellation);
    assert!(dut.is_navigation_rinex(), "should be declared as NAV RINEX");
    /*
     * Header specific fields
     */
    assert!(
        dut.header.obs.is_none(),
        "should not contain specific OBS fields"
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
}

/*
 * Any parsed CLOCK RINEX should go through this test
 */
pub fn test_clock_rinex(dut: &Rinex, version: &str, constellation: Option<&str>) {
    test_rinex(dut, version, constellation);
    assert!(dut.is_clock_rinex(), "should be declared as CLK RINEX");
    /*
     * Header specific fields
     */
    assert!(
        dut.header.obs.is_none(),
        "should not contain specific OBS fields"
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
        dut.header.clock.is_some(),
        "should contain specific CLOCK fields"
    );
}

/*
 * Any parsed IONEX should go through this test
 */
pub fn test_ionex(dut: &Rinex, version: &str, constellation: Option<&str>) {
    test_rinex(dut, version, constellation);
    assert!(dut.is_ionex(), "should be declared as IONEX");
    /*
     * Header specific fields
     */
    assert!(
        dut.header.obs.is_none(),
        "should not contain specific OBS fields"
    );
    assert!(
        dut.header.meteo.is_none(),
        "should not contain specific METEO fields"
    );
    assert!(
        dut.header.ionex.is_some(),
        "should contain specific IONEX fields"
    );
    assert!(
        dut.header.clock.is_none(),
        "should not contain specific CLOCK fields"
    );
}
