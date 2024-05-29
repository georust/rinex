#[cfg(test)]
mod test {
    use crate::navigation::NavMsgType;
    use crate::prelude::*;
    use crate::tests::toolkit::is_null_rinex;
    use std::path::PathBuf;
    #[test]
    fn test_parser() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("../test_resources");
        let test_data = vec!["ATX", "CLK", "CRNX", "MET", "NAV", "OBS", "IONEX"];
        for data in test_data {
            let data_path = test_resources.clone().join(data);
            for revision in std::fs::read_dir(data_path).unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let revision = rev_path.file_stem().unwrap().to_string_lossy().to_string();
                let rev_fullpath = &rev_path.to_str().unwrap();
                for entry in std::fs::read_dir(rev_fullpath).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry.file_name().to_str().unwrap().starts_with('.');
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
                    println!("Parsing \"{}\"", full_path);
                    let rinex = Rinex::from_file(full_path);
                    assert!(
                        rinex.is_ok(),
                        "error parsing \"{}\": {:?}",
                        full_path,
                        rinex.err().unwrap()
                    );
                    let rinex = rinex.unwrap();

                    match data {
                        "ATX" => {
                            assert!(rinex.is_antex());
                        },
                        "NAV" => {
                            assert!(rinex.is_navigation_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                            assert!(rinex.navigation().count() > 0); // all files have content
                                                                     // Ephemeris verifications
                            for (e, (_, sv_i, eph_i)) in rinex.ephemeris() {
                                // TODO: verify V4 cases
                                if revision != "V4" {
                                    // Verify week counter
                                    match sv_i.constellation {
                                        Constellation::GPS
                                        | Constellation::Galileo
                                        | Constellation::BeiDou => {
                                            assert!(
                                                eph_i.get_week().is_some(),
                                                "should have week counter: {:?}",
                                                eph_i.orbits
                                            );
                                        },
                                        c => {
                                            if c.is_sbas() {
                                                assert!(
                                                    eph_i.get_week().is_some(),
                                                    "should have week counter: {:?}",
                                                    eph_i.orbits
                                                );
                                            }
                                        },
                                    }
                                }
                            }
                        },
                        "CRNX" | "OBS" => {
                            assert!(rinex.header.obs.is_some());
                            let obs_header = rinex.header.obs.clone().unwrap();

                            assert!(rinex.is_observation_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                            assert!(rinex.observation().count() > 0); // all files have content
                            is_null_rinex(&rinex.substract(&rinex), 1.0E-9); // Self - Self should always be null
                            if data == "OBS" {
                                let compressed = rinex.rnx2crnx();
                                assert!(
                                    compressed.header.is_crinex(),
                                    "is_crinex() should always be true for compressed rinex!"
                                );
                            } else if data == "CRNX" {
                                let decompressed = rinex.crnx2rnx();
                                assert!(
                                    !decompressed.header.is_crinex(),
                                    "is_crinex() should always be false for readable rinex!"
                                );
                            }

                            /* Timescale validity */
                            for ((e, _), _) in rinex.observation() {
                                let ts = e.time_scale;
                                if let Some(e0) = obs_header.time_of_first_obs {
                                    assert!(
                                        e0.time_scale == ts,
                                        "interpreted wrong timescale: expecting \"{}\", got \"{}\"",
                                        e0.time_scale,
                                        ts
                                    );
                                } else {
                                    match rinex.header.constellation {
                                        Some(Constellation::Mixed) | None => {}, // can't test
                                        Some(c) => {
                                            let timescale = c.timescale().unwrap();
                                            assert!(ts == timescale,
                                                "interpreted wrong timescale: expecting \"{}\", got \"{}\"",
                                                timescale,
                                                ts
                                            );
                                        },
                                    }
                                }
                            }
                        },
                        "MET" => {
                            assert!(rinex.is_meteo_rinex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                            assert!(rinex.meteo().count() > 0); // all files have content
                            for (e, _) in rinex.meteo() {
                                assert!(
                                    e.time_scale == TimeScale::UTC,
                                    "wrong {} time scale for a METEO RINEX",
                                    e.time_scale
                                );
                            }
                        },
                        "CLK" => {
                            assert!(rinex.is_clock_rinex(), "badly identified CLK RINEX");
                            assert!(rinex.header.clock.is_some(), "badly formed CLK RINEX");
                            assert!(rinex.epoch().count() > 0); // all files have content
                            let record = rinex.record.as_clock().unwrap();
                        },
                        "IONEX" => {
                            assert!(rinex.is_ionex());
                            assert!(rinex.epoch().count() > 0); // all files have content
                            for e in rinex.epoch() {
                                assert!(
                                    e.time_scale == TimeScale::UTC,
                                    "wrong {} timescale for a IONEX",
                                    e.time_scale
                                );
                            }
                        },
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}
