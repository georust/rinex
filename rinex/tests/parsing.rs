#[cfg(test)]
mod test {
    use rinex::prelude::*;
    #[test]
    fn test_parser() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let test_data = vec!["ATX", "CLK", "CRNX", "MET", "NAV", "OBS", "IONEX"];
        for data in test_data {
            let data_path = std::path::PathBuf::from(test_resources.to_owned() + data);
            for revision in std::fs::read_dir(data_path).unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let rev_fullpath = &rev_path.to_str().unwrap();
                for entry in std::fs::read_dir(rev_fullpath).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry.file_name().to_str().unwrap().starts_with(".");
                    if is_hidden {
                        continue; // not a test resource
                    }
                    let is_generated_file = entry.file_name().to_str().unwrap().ends_with("-copy");
                    if is_generated_file {
                        continue; // not a test resource
                    }

                    let mut is_gzip_encoded = entry.file_name().to_str().unwrap().ends_with(".gz");
                    is_gzip_encoded |= entry.file_name().to_str().unwrap().ends_with(".Z");
                    if is_gzip_encoded && !cfg!(feature = "flate2") {
                        continue; // do not run in this build configuration
                    }
                    println!("Parsing file: \"{}\"", full_path);
                    let rinex = Rinex::from_file(full_path);
                    assert_eq!(rinex.is_ok(), true);
                    let rinex = rinex.unwrap();

                    match data {
                        "ATX" => {
                            assert!(rinex.is_antex_rinex());
                        },
                        "NAV" => {
                            assert!(rinex.is_navigation_rinex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                        },
                        "OBS" => {
                            assert!(rinex.header.obs.is_some());
                            assert!(rinex.is_observation_rinex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                            /*
                                                        let gf = rinex.observation_gf_combinations();
                                                        let nl = rinex.observation_nl_combinations();
                                                        let wl = rinex.observation_wl_combinations();
                                                        let mw = rinex.observation_mw_combinations();

                                                        let mut gf_combinations: Vec<_> = gf.keys().collect();
                                                        let mut nl_combinations: Vec<_> = nl.keys().collect();
                                                        let mut wl_combinations: Vec<_> = wl.keys().collect();
                                                        let mut mw_combinations: Vec<_> = mw.keys().collect();

                                                        gf_combinations.sort();
                                                        nl_combinations.sort();
                                                        wl_combinations.sort();
                                                        mw_combinations.sort();

                                                        assert_eq!(gf_combinations, nl_combinations);
                                                        assert_eq!(gf_combinations, wl_combinations);
                                                        assert_eq!(gf_combinations, mw_combinations);

                                                        assert_eq!(nl_combinations, wl_combinations);
                                                        assert_eq!(nl_combinations, mw_combinations);

                                                        assert_eq!(wl_combinations, mw_combinations);
                            */
                        },
                        "CRNX" => {
                            assert!(rinex.header.obs.is_some());
                            assert!(rinex.is_observation_rinex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                        },
                        "MET" => {
                            //assert_eq!(rinex.header.obs.is_some(), true);
                            assert!(rinex.is_meteo_rinex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                        },
                        "CLK" => {
                            assert!(rinex.is_clocks_rinex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                        },
                        "IONEX" => {
                            assert!(rinex.is_ionex());
                            assert!(rinex.epoch().collect::<Vec<Epoch>>().len() > 0);
                        },
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
