#[cfg(test)]
mod test {
    use rinex::navigation::NavMsgType;
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
                            assert!(rinex.is_antex());
                        },
                        "NAV" => {
                            assert!(rinex.is_navigation_rinex());
                            assert!(rinex.epoch().next().is_some());
                            assert!(rinex.epoch().count() > 0); // all files have content
                                                                /*
                                                                 * Verify ION logical correctness
                                                                 */
                            for (_, (msg, sv, ion_msg)) in rinex.ionosphere_models() {
                                if sv.constellation == Constellation::GPS {
                                    assert!(
                                        ion_msg.as_klobuchar().is_some(),
                                        "only Kb models provided by GPS vehicles"
                                    );
                                } else if sv.constellation == Constellation::QZSS {
                                    assert!(
                                        ion_msg.as_klobuchar().is_some(),
                                        "only Kb models provided by QZSS vehicles"
                                    );
                                } else if sv.constellation == Constellation::BeiDou {
                                    match msg {
                                        NavMsgType::D1D2 => {
                                            assert!(
                                                ion_msg.as_klobuchar().is_some() == true,
                                                "BeiDou ({}) should be interpreted as Kb model",
                                                msg
                                            );
                                        },
                                        NavMsgType::CNVX => {
                                            assert!(
                                                ion_msg.as_bdgim().is_some() == true,
                                                "BeiDou (CNVX) should be interpreted as Bd model"
                                            );
                                        },
                                        _ => {
                                            panic!(
                                                "invalid message type \"{}\" for BeiDou ION frame",
                                                msg
                                            );
                                        },
                                    }
                                } else if sv.constellation == Constellation::IRNSS {
                                    assert!(
                                        ion_msg.as_klobuchar().is_some(),
                                        "only Kb models provided by NavIC/IRNSS vehicles"
                                    );
                                } else if sv.constellation == Constellation::Galileo {
                                    assert!(
                                        ion_msg.as_nequick_g().is_some(),
                                        "only Ng models provided by GAL vehicles"
                                    );
                                } else {
                                    panic!(
                                        "incorrect constellation provider of an ION model: {}",
                                        sv.constellation
                                    );
                                }
                            }
                            /*
                             * Verify EOP logical correctness
                             */
                            for (_, (msg, sv, _)) in rinex.earth_orientation() {
                                assert!(
                                    (sv.constellation == Constellation::GPS)
                                    || (sv.constellation == Constellation::QZSS)
                                    || (sv.constellation == Constellation::IRNSS)
                                    || (sv.constellation == Constellation::BeiDou), "constellation \"{}\" not declared as EOP frame provider, according to V4 specs", sv.constellation);
                                assert!(
                                    (msg == NavMsgType::CNVX) || (msg == NavMsgType::LNAV),
                                    "bad msg identified for GPS vehicle: {}",
                                    msg
                                );
                            }
                            /*
                             * Verify STO logical correctness
                             */
                            for (_, (msg, sv, _)) in rinex.system_time_offset() {
                                assert!(
                                    (msg == NavMsgType::LNAV)
                                        || (msg == NavMsgType::FDMA)
                                        || (msg == NavMsgType::IFNV)
                                        || (msg == NavMsgType::D1D2)
                                        || (msg == NavMsgType::SBAS)
                                        || (msg == NavMsgType::CNVX),
                                    "bad \"{}\" message for STO frame",
                                    msg
                                );
                            }
                        },
                        "OBS" => {
                            assert!(rinex.header.obs.is_some());
                            assert!(rinex.is_observation_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
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
                            assert!(rinex.epoch().count() > 0); // all files have content
                        },
                        "MET" => {
                            assert!(rinex.is_meteo_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                        },
                        "CLK" => {
                            assert!(rinex.is_clocks_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                        },
                        "IONEX" => {
                            assert!(rinex.is_ionex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                        },
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
