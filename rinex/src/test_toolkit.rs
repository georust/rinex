use crate::*;
/*
 * OBS RINEX thorough comparison
 */
fn observation_comparison(dut: &Rinex, model: &Rinex, filename: &str) {
    let rec_dut = dut
        .record
        .as_obs()
        .expect("failed to unwrap as observation rinex record");
    let rec_model = model
        .record
        .as_obs()
        .expect("failed to unwrap as observation rinex record");

    for (e_model, (clk_offset_model, vehicules_model)) in rec_model.iter() {
        if let Some((clk_offset_dut, vehicules_dut)) = rec_dut.get(e_model) {
            assert_eq!(
                clk_offset_model, clk_offset_dut,
                "\"{}\" - {:?} - faulty clock offset, expecting {:?} got {:?}",
                filename, e_model, clk_offset_model, clk_offset_dut
            );
            for (sv_model, observables_model) in vehicules_model.iter() {
                if let Some(observables_dut) = vehicules_dut.get(sv_model) {
                    for (code_model, obs_model) in observables_model {
                        if let Some(obs_dut) = observables_dut.get(code_model) {
                            assert!(
                                (obs_model.obs - obs_dut.obs).abs() < 1.0E-6,
                                "\"{}\" - {:?} - {:?} - \"{}\" expecting {} got {}",
                                filename,
                                e_model,
                                sv_model,
                                code_model,
                                obs_model.obs,
                                obs_dut.obs
                            );
                            assert_eq!(
                                obs_model.lli, obs_dut.lli,
                                "\"{}\" - {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}",
                                filename, e_model, sv_model, code_model, obs_model.lli, obs_dut.lli
                            );
                            assert_eq!(
                                obs_model.snr, obs_dut.snr,
                                "\"{}\" - {:?} - {:?} - \"{}\" - SNR expecting {:?} got {:?}",
                                filename, e_model, sv_model, code_model, obs_model.snr, obs_dut.snr
                            );
                        } else {
                            panic!(
                                "\"{}\" - {:?} - {:?} : missing \"{}\" observation",
                                filename, e_model, sv_model, code_model
                            );
                        }
                    }
                } else {
                    panic!(
                        "\"{}\" - {:?} - missing vehicule {:?}",
                        filename, e_model, sv_model
                    );
                }
            }
        } else {
            panic!("\"{}\" - missing epoch {:?}", filename, e_model);
        }
    }

    for (e_b, (clk_offset_b, vehicules_b)) in rec_model.iter() {
        if let Some((clk_offset_model, vehicules_model)) = rec_dut.get(e_b) {
            assert_eq!(clk_offset_model, clk_offset_b);
            for (sv_b, observables_b) in vehicules_b.iter() {
                if let Some(observables_model) = vehicules_model.get(sv_b) {
                    for (code_b, obs_b) in observables_b {
                        if let Some(obs_model) = observables_model.get(code_b) {
                            assert!(
                                (obs_model.obs - obs_b.obs).abs() < 1.0E-6,
                                "\"{}\" - {:?} - {:?} - \"{}\" expecting {} got {}",
                                filename,
                                e_b,
                                sv_b,
                                code_b,
                                obs_model.obs,
                                obs_b.obs
                            );
                            assert_eq!(
                                obs_model.lli, obs_b.lli,
                                "\"{}\" - {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}",
                                filename, e_b, sv_b, code_b, obs_model.lli, obs_b.lli
                            );
                            assert_eq!(
                                obs_model.snr, obs_b.snr,
                                "\"{}\" - {:?} - {:?} - \"{}\" - SNR expecting {:?} got {:?}",
                                filename, e_b, sv_b, code_b, obs_model.snr, obs_b.snr
                            );
                        } else {
                            panic!(
                                "\"{}\" - {:?} - {:?} : parsed \"{}\" unexpectedly",
                                filename, e_b, sv_b, code_b
                            );
                        }
                    }
                } else {
                    panic!(
                        "\"{}\" - {:?} - parsed {:?} unexpectedly",
                        filename, e_b, sv_b
                    );
                }
            }
        } else {
            panic!("\"{}\" - parsed epoch {:?} unexpectedly", filename, e_b);
        }
    }
}

/*
 * CLOCK Rinex thorough comparison
 */
fn clocks_comparison(dut: &Rinex, model: &Rinex, filename: &str) {
    let rec_dut = dut
        .record
        .as_clock()
        .expect("failed to unwrap as clock rinex record");
    let rec_model = model
        .record
        .as_clock()
        .expect("failed to unwrap as clock rinex record");
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
 * Meteo RINEX thorough comparison
 */
fn meteo_comparison(dut: &Rinex, model: &Rinex, filename: &str) {
    let rec_dut = dut
        .record
        .as_meteo()
        .expect("failed to unwrap as meteo rinex record");
    let rec_model = model
        .record
        .as_meteo()
        .expect("failed to unwrap as meteo rinex record");
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
pub fn compare_with_panic(dut: &Rinex, model: &Rinex, filename: &str) {
    if dut.is_observation_rinex() {
        observation_comparison(&dut, &model, filename);
    } else if dut.is_meteo_rinex() {
        meteo_comparison(&dut, &model, filename);
    } else if dut.is_clocks_rinex() {
        clocks_comparison(&dut, &model, filename);
    }
}
