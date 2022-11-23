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
                                assert!((obs_a.obs - obs_b.obs).abs() < 1.0E-6, 
                                    "epoch {:?} - {:?} - \"{}\" expecting {} got {}", e_a, sv_a, code_a, obs_b.obs, obs_a.obs);
                                assert_eq!(obs_a.lli, obs_b.lli,
                                    "epoch {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}", e_a, sv_a, code_a, obs_b.lli, obs_a.lli);
                                assert_eq!(obs_a.ssi, obs_b.ssi,
                                    "epoch {:?} - {:?} - \"{}\" - SSI expecting {:?} got {:?}", e_a, sv_a, code_a, obs_b.ssi, obs_a.ssi);
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
                                assert!((obs_a.obs - obs_b.obs).abs() < 1.0E-6, 
                                    "epoch {:?} - {:?} - \"{}\" expecting {} got {}", e_b, sv_b, code_b, obs_b.obs, obs_a.obs);
                                assert_eq!(obs_a.lli, obs_b.lli,
                                    "epoch {:?} - {:?} - \"{}\" - LLI expecting {:?} got {:?}", e_b, sv_b, code_b, obs_b.lli, obs_a.lli);
                                assert_eq!(obs_a.ssi, obs_b.ssi,
                                    "epoch {:?} - {:?} - \"{}\" - SSI expecting {:?} got {:?}", e_b, sv_b, code_b, obs_b.ssi, obs_a.ssi);
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
            //("zegv0010.21d", "zegv0010.21o"),
            //("AJAC3550.21D", "AJAC3550.21O"), 
            ("aopr0010.17d", "aopr0010.17o"),
            //("npaz3550.21d", "npaz3550.21o"),
            //("pdel0010.21d", "pdel0010.21o"),
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
                assert_eq!(infos.date, Epoch::from_gregorian_utc(2021, 01, 02, 00, 01, 00, 00));
            } else if crnx_name.eq("npaz3550.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, Epoch::from_gregorian_utc(2021, 12, 28, 00, 18, 00, 00));
            } else if crnx_name.eq("pdel0010.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, Epoch::from_gregorian_utc(2021, 01, 09, 00, 24, 00, 00));
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
    #[test]
    fn testbench_v3() {
        let pool = vec![
            ("ACOR00ESP_R_20213550000_01D_30S_MO.crx","ACOR00ESP_R_20213550000_01D_30S_MO.rnx"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse CRINEX
            let path = format!("../test_resources/CRNX/V3/{}", crnx_name);
            let crnx = Rinex::from_file(&path);
            
            assert_eq!(crnx.is_ok(), true);
            let mut rnx = crnx.unwrap();
            assert_eq!(rnx.header.obs.is_some(), true);
            let obs = rnx.header.obs.as_ref().unwrap();
            assert_eq!(obs.crinex.is_some(), true);
            let infos = obs.crinex.as_ref().unwrap();

            if crnx_name.eq("ACOR00ESP_R_20213550000_01D_30S_MO.crx") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(infos.date, Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00));
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
                &format!("../test_resources/OBS/V3/{}", rnx_name),
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
        let mut decompressor = Decompressor::new();
		assert!(decompressor.decompress(1, &Constellation::Mixed, 2, &obscodes, content).is_err());
    }
	#[test]
	fn zegv0010_21d() {
		let rnx = Rinex::from_file("../test_resources/CRNX/V1/zegv0010.21d")
			.unwrap();
		let epochs = vec![
			Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 00, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 01, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 01, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 02, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 02, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 03, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 03, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 04, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 04, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 05, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 05, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 06, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 06, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 07, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 07, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 08, 00, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 08, 30, 00),
            Epoch::from_gregorian_utc(2021, 01, 01, 00, 09, 00, 00),
        ];
		assert_eq!(rnx.epochs(), epochs);
		let record = rnx.record.as_obs().unwrap();

		for (index, ((e, flag), (clk_offset, vehicules))) in record.iter().enumerate() {
			assert!(flag.is_ok());
			assert!(clk_offset.is_none());
			if index == 0 {
				assert_eq!(vehicules.len(), 24);
				for (sv, observations) in vehicules {
					if *sv == Sv::new(Constellation::GPS, 07) {
						let mut keys: Vec<_> = observations.keys().collect();
						keys.sort();
						assert_eq!(keys,
							vec!["C1","C2", "L1","L2","P1","P2","S1","S2"]);
						let c1 = observations.get("C1")
							.unwrap();
						assert_eq!(c1.obs, 24178026.635);
						let c2 = observations.get("C2")
							.unwrap();
						assert_eq!(c2.obs, 24178024.891);
						let l1 = observations.get("L1")
							.unwrap();
						assert_eq!(l1.obs, 127056391.699); 
						let l2 = observations.get("L2")
							.unwrap();
						assert_eq!(l2.obs, 99004963.017); 
						let p1 = observations.get("P1")
							.unwrap();
						assert_eq!(p1.obs, 24178026.139); 
						let p2 = observations.get("P2")
							.unwrap();
						assert_eq!(p2.obs, 24178024.181); 
						let s1 = observations.get("S1")
							.unwrap();
						assert_eq!(s1.obs, 38.066); 
						let s2 = observations.get("S2")
							.unwrap();
						assert_eq!(s2.obs, 22.286); 
					} else if *sv == Sv::new(Constellation::GPS, 08) {
						let mut keys: Vec<_> = observations.keys().collect();
						keys.sort();
						assert_eq!(keys,
							vec!["C1","C2","C5", "L1","L2","L5","P1","P2","S1","S2","S5"]);
						let c1 = observations.get("C1")
							.unwrap();
						assert_eq!(c1.obs, 21866748.928);
						let c2 = observations.get("C2")
							.unwrap();
						assert_eq!(c2.obs, 21866750.407);
						let c5 = observations.get("C5")
							.unwrap();
						assert_eq!(c5.obs, 21866747.537);
						let l1 = observations.get("L1")
							.unwrap();
						assert_eq!(l1.obs,114910552.082 ); 
						let l2 = observations.get("L2")
							.unwrap();
						assert_eq!(l2.obs, 89540700.326); 
						let l5 = observations.get("L5")
							.unwrap();
						assert_eq!(l5.obs, 85809828.276);
						let p1 = observations.get("P1")
							.unwrap();
						assert_eq!(p1.obs, 21866748.200); 
						let p2 = observations.get("P2")
							.unwrap();
						assert_eq!(p2.obs, 21866749.482); 
						let s1 = observations.get("S1")
							.unwrap();
						assert_eq!(s1.obs, 45.759); 
						let s2 = observations.get("S2")
							.unwrap();
						assert_eq!(s2.obs, 49.525); 
						let s5 = observations.get("S5")
							.unwrap();
						assert_eq!(s5.obs, 52.161);
					} else if *sv == Sv::new(Constellation::GPS, 13) {
						let mut keys: Vec<_> = observations.keys().collect();
						keys.sort();
						assert_eq!(keys,
							vec!["C1", "L1","L2","P1","P2","S1","S2"]);
						//let c1 = observations.get("C1")
						//	.unwrap();
						//assert_eq!(s2.obs, 49.525); 
//  25107711.730 5                                 131941919.38305 102811868.09001
//                  25107711.069 1  25107709.586 1        33.150           8.952  
						let c1 = observations.get("C1").unwrap();
						assert_eq!(c1.obs, 25107711.730);
						let l1 = observations.get("L1").unwrap();
						assert_eq!(l1.obs, 131941919.383);
						let l2 = observations.get("L2").unwrap();
						assert_eq!(l2.obs, 102811868.090);
						let p1 = observations.get("P1").unwrap();
						assert_eq!(p1.obs, 25107711.069);
						let p2 = observations.get("P2").unwrap();
						assert_eq!(p2.obs, 25107709.586);
						let s1 = observations.get("S1").unwrap();
						assert_eq!(s1.obs, 33.150);
						let s2 = observations.get("S2").unwrap();
						assert_eq!(s2.obs, 8.952);
					}
				}
			}
		}
	}
    #[test]
    fn acor00esp_r_2021_crx() {
        let crnx = Rinex::from_file("../test_resources/CRNX/V3/ACOR00ESP_R_20213550000_01D_30S_MO.crx");
        assert_eq!(crnx.is_ok(), true);
        let mut rnx = crnx.unwrap();
        
        assert_eq!(rnx.header.obs.is_some(), true);
        let obs = rnx.header.obs.as_ref().unwrap();
        assert_eq!(obs.crinex.is_some(), true);
        let infos = obs.crinex.as_ref().unwrap();

        assert_eq!(infos.version.major, 3);
        assert_eq!(infos.version.minor, 0);
        assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
        assert_eq!(infos.date, Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00));

        let epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,00,  0, 0),
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,00, 30, 0),
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,01,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,01, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,02,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,02, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,03,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,03, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,04,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,04, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,05,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,05, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,06,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,06, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,07,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,07, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,08,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,08, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,09,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,09, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,10,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,10, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,11,  0, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,11, 30, 0) ,
            Epoch::from_gregorian_utc(2021, 12, 21, 00 ,12,  0, 0) ,
        ];
        assert_eq!(rnx.epochs(), epochs);
    }
}
