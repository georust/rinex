#[cfg(test)]
mod test {
    use rinex::{
        prelude::*,
        version::Version,
        hatanaka::Decompressor,
        observation::{HeaderFields, Crinex},
    };
    use std::collections::HashMap;
    /*
     * Compares rnx_b to rnx_a 
     */
    fn run_comparison(rnx_a: &str, rnx_b: &str) {
        let rnx_a = Rinex::from_file(rnx_a);
        assert_eq!(rnx_a.is_ok(), true);
        let rnx_a = rnx_a.unwrap();
        let rec_a = rnx_a.record.as_obs().unwrap();
        
        let rnx_b = Rinex::from_file(rnx_b);
        assert_eq!(rnx_b.is_ok(), true);
        let rnx_b = rnx_b.unwrap();
        let rec_b = rnx_b.record.as_obs().unwrap();

        for (e_a, (clk_offset_a, vehicules_a)) in rec_a.iter() {
            if let Some((clk_offset_b, vehicules_b)) = rec_b.get(e_a) {
                assert_eq!(clk_offset_a, clk_offset_b);
                for (sv_a, observables_a) in vehicules_a.iter() {
                    if let Some(observables_b) = vehicules_b.get(sv_a) {
                        for (code_a, obs_a) in observables_a {
                            if let Some(obs_b) = observables_b.get(code_a) {
                                assert!((obs_a.obs - obs_b.obs).abs() < 1E-6);
                                assert_eq!(obs_a.lli, obs_b.lli);
                                assert_eq!(obs_a.ssi, obs_b.ssi);
                            } else {
                                panic!("epoch {:?} - {:?} : missing \"{}\" observation", e_a, sv_a, code_a);
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
                                assert!((obs_a.obs - obs_b.obs).abs() < 1E-6);
                                assert_eq!(obs_a.lli, obs_b.lli);
                                assert_eq!(obs_a.ssi, obs_b.ssi);
                            } else {
                                panic!("epoch {:?} - {:?} : parsed \"{}\" unexpectedly", e_b, sv_b, code_b);
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
    #[test]
    fn testbench_v1() {
        let pool = vec![
            ("zegv0010.21d", "zegv0010.21o"),
            //("AJAC3550.21D", "AJAC3550.21O"), 
            //("aopr0010.17d", "aopr0010.17o"),
            ("npaz3550.21d", "npaz3550.21o"),
            ("pdel0010.21d", "pdel0010.21o"),
            //("wsra0010.21d", "wsra0010.21o"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse CRINEX
            let path = format!("../test_resources/CRNX/V1/{}", crnx_name);
            let crnx = Rinex::from_file(&path);
            
            assert_eq!(crnx.is_ok(), true);
            let mut rnx = crnx.unwrap();
            assert_eq!(rnx.header.obs.is_some(), true);
            let obs = rnx.header.obs.as_ref().unwrap();
            assert_eq!(obs.crinex.is_some(), true);
            let infos = obs.crinex.as_ref().unwrap();

            if crnx_name.eq("zegv0010.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, hifitime::Epoch::from_gregorian_utc(2021, 01, 02, 00, 01, 00, 00));
            } else if crnx_name.eq("npaz3550.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, hifitime::Epoch::from_gregorian_utc(2021, 12, 28, 00, 18, 00, 00));
            } else if crnx_name.eq("pdel0010.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, hifitime::Epoch::from_gregorian_utc(2021, 01, 09, 00, 24, 00, 00));
            }

            // convert to RINEX
            rnx.crnx2rnx();
            
            let obs = rnx.header.obs.as_ref().unwrap();
            assert_eq!(obs.crinex.is_some(), false);

            // dump to file
            let rnx_b_path = format!("test-{}", rnx_name);
            assert_eq!(rnx.to_file(&rnx_b_path).is_ok(), true);
            // run testbench
            run_comparison(
                &format!("../test_resources/OBS/V2/{}", rnx_name),
                &rnx_b_path);
            //let _ = std::fs::remove_file(&rnx_b_path);
        }
    }
    /*
     * Tries decompression against faulty CRINEX1 content
     */
    #[test]
    fn test_faulty_crinex1() {
        let mut obscodes: HashMap<Constellation, Vec<String>>
            = HashMap::new();
        obscodes.insert(
            Constellation::GPS,
            vec![
                String::from("L1"),
                String::from("L2"),
                String::from("C1"),
                String::from("P2"),
                String::from("P1"),
                String::from("S1"),
                String::from("S2")
            ]);
        obscodes.insert(
            Constellation::Glonass,
            vec![
                String::from("L1"),
                String::from("L2"),
                String::from("C1"),
                String::from("P2"),
                String::from("P1"),
                String::from("S1"),
                String::from("S2")
            ]);
        let content = "21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15";
        let header = Header::basic_obs()
            .with_version(Version {
                major: 2,
                minor: 11,
            })
            .with_constellation(Constellation::Mixed)
            .with_observation_fields(HeaderFields {
                codes: obscodes.clone(), 
                crinex: Some(Crinex {
                    version: Version {
                        major: 1,
                        minor: 0,
                    },
                    prog: "testing".to_string(),
                    date: hifitime::Epoch::now().unwrap(),
                }),
                clock_offset_applied: false,
                dcb_compensations: Vec::new(),
                scalings: HashMap::new(),
            });
        let mut decompressor = Decompressor::new();
        let decompressed = decompressor.decompress(&header, content);
        assert_eq!(decompressed.is_err(), true);
    }
}
