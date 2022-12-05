#[cfg(test)]
mod test {
    use rinex::*;
    /// OBS RINEX thorough comparison
    fn observation_comparison(rnx_a: &Rinex, rnx_b: &Rinex) {
        let rec_a = rnx_a.record.as_obs().unwrap();
        let rec_b = rnx_b.record.as_obs().unwrap();
        for (e_a, (clk_offset_a, vehicules_a)) in rec_a.iter() {
            if let Some((clk_offset_b, vehicules_b)) = rec_b.get(e_a) {
                assert_eq!(clk_offset_a, clk_offset_b);
                for (sv_a, observables_a) in vehicules_a.iter() {
                    if let Some(observables_b) = vehicules_b.get(sv_a) {
                        for (code_a, obs_a) in observables_a {
                            if let Some(obs_b) = observables_b.get(code_a) {
                                assert!(
                                    (obs_a.obs - obs_b.obs).abs() < 1.0E-6,
                                    "epoch {:?} - {:?} - \"{}\" expecting {} got {}",
                                    e_a,
                                    sv_a,
                                    code_a,
                                    obs_b.obs,
                                    obs_a.obs
                                );
                                assert_eq!(
                                    obs_a.lli, obs_b.lli,
                                    "epoch {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}",
                                    e_a, sv_a, code_a, obs_b.lli, obs_a.lli
                                );
                                assert_eq!(
                                    obs_a.ssi, obs_b.ssi,
                                    "epoch {:?} - {:?} - \"{}\" - SSI expecting {:?} got {:?}",
                                    e_a, sv_a, code_a, obs_b.ssi, obs_a.ssi
                                );
                            } else {
                                panic!(
                                    "epoch {:?} - {:?} : missing \"{}\" observation",
                                    e_a, sv_a, code_a
                                );
                            }
                        }
                    } else {
                        panic!("epoch {:?} - missing vehicule {:?}", e_a, sv_a);
                    }
                }
            } else {
                panic!("missing epoch {:?}", e_a);
            }
        }

        for (e_b, (clk_offset_b, vehicules_b)) in rec_b.iter() {
            if let Some((clk_offset_a, vehicules_a)) = rec_a.get(e_b) {
                assert_eq!(clk_offset_a, clk_offset_b);
                for (sv_b, observables_b) in vehicules_b.iter() {
                    if let Some(observables_a) = vehicules_a.get(sv_b) {
                        for (code_b, obs_b) in observables_b {
                            if let Some(obs_a) = observables_a.get(code_b) {
                                assert!(
                                    (obs_a.obs - obs_b.obs).abs() < 1.0E-6,
                                    "epoch {:?} - {:?} - \"{}\" expecting {} got {}",
                                    e_b,
                                    sv_b,
                                    code_b,
                                    obs_b.obs,
                                    obs_a.obs
                                );
                                assert_eq!(
                                    obs_a.lli, obs_b.lli,
                                    "epoch {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}",
                                    e_b, sv_b, code_b, obs_b.lli, obs_a.lli
                                );
                                assert_eq!(
                                    obs_a.ssi, obs_b.ssi,
                                    "epoch {:?} - {:?} - \"{}\" - SSI expecting {:?} got {:?}",
                                    e_b, sv_b, code_b, obs_b.ssi, obs_a.ssi
                                );
                            } else {
                                panic!(
                                    "epoch {:?} - {:?} : parsed \"{}\" unexpectedly",
                                    e_b, sv_b, code_b
                                );
                            }
                        }
                    } else {
                        panic!("epoch {:?} - parsed {:?} unexpectedly", e_b, sv_b);
                    }
                }
            } else {
                panic!("parsed epoch {:?} unexpectedly", e_b);
            }
        }
    }

    /// CLOCK Rinex thorough comparison
    fn clocks_comparison(rnx_a: &Rinex, rnx_b: &Rinex) {
        let rec_a = rnx_a.record.as_clock().unwrap();
        let rec_b = rnx_a.record.as_clock().unwrap();
        for (e_a, data_types) in rec_a.iter() {
            for (data_type, systems) in rec_a.iter() {
                for (system, data) in systems.iter() {}
            }
        }
    }

    /// Meteo RINEX thorough comparison
    fn meteo_comparison(rnx_a: &Rinex, rnx_b: &Rinex) {
        let rec_a = rnx_a.record.as_meteo().unwrap();
        let rec_b = rnx_b.record.as_meteo().unwrap();
        for (e_a, obscodes_a) in rec_a.iter() {
            if let Some(obscodes_b) = rec_b.get(e_a) {
                for (code_a, observation_a) in obscodes_a.iter() {
                    if let Some(observation_b) = obscodes_b.get(code_a) {
                        assert_eq!(observation_a, observation_b);
                    } else {
                        panic!("epoch {:?} missing \"{}\" observation", e_a, code_a);
                    }
                }
            } else {
                panic!("missing epoch {:?}", e_a);
            }
        }

        for (e_b, obscodes_b) in rec_b.iter() {
            if let Some(obscodes_a) = rec_a.get(e_b) {
                for (code_b, observation_b) in obscodes_b.iter() {
                    if let Some(observation_a) = obscodes_a.get(code_b) {
                        assert_eq!(observation_a, observation_b);
                    } else {
                        panic!("epoch {:?} parsed \"{}\" unexpectedly", e_b, code_b);
                    }
                }
            } else {
                panic!("parsed {:?} unexpectedly", e_b);
            }
        }
    }
    fn compare_with_panic(rnx_a: &Rinex, rnx_b: &Rinex) {
        if rnx_a.is_observation_rinex() {
            observation_comparison(&rnx_a, &rnx_b);
        } else if rnx_a.is_meteo_rinex() {
            meteo_comparison(&rnx_a, &rnx_b);
        } else if rnx_a.is_clocks_rinex() {
            clocks_comparison(&rnx_a, &rnx_b);
        }
    }
    fn testbench(path: &str) {
        println!("Running testbench on: \"{}\"", path);
        let rnx_a = Rinex::from_file(path).unwrap(); // tested in parser dedicated testsuite
                                                     // generate a copy
        let copy_path = path.to_owned() + "-copy";
        assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
        // parse copy
        let rnx_b = Rinex::from_file(&copy_path);
        assert_eq!(rnx_b.is_ok(), true);
        let rnx_b = rnx_b.unwrap();
        if rnx_a != rnx_b {
            compare_with_panic(&rnx_a, &rnx_b);
        }
        // remove copy not to disturb other test browsers
        let _ = std::fs::remove_file(copy_path);
        // sleeping here for a bit,
        // avoids this (temporary) file being picked up by other automated tests
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    #[test]
    fn obs_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn obs_v3() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn meteo_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn meteo_v4() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn clocks_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/CLK/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn nav_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    fn nav_v3() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    /*
    #[test]
    fn nav_v4() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }*/
}
