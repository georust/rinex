#[cfg(test)]
mod test {
    use rinex::*;
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
                                assert_eq!(obs_a, obs_b);
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
                                assert_eq!(obs_a, obs_b);
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
    //#[test]
    fn testbench_v1() {
        let pool = vec![
            ("zegv0010.21d", "zegv0010.21o"),
            //("AJAC3550.21D", "AJAC3550.21O"), 
            //("aopr0010.17d", "aopr0010.17o"),
            //("npaz3550.21d", "npaz3550.21o"),
            //("pdel0010.21d", "pdel0010.21o"),
            //("wsra0010.21d", "wsra0010.21o"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse RINEX
            let path = format!("../test_resources/OBS/V2/{}", rnx_name);
            let rnx = Rinex::from_file(&path);
            assert_eq!(rnx.is_ok(), true);
            let mut crnx = rnx.unwrap();
            // convert to CRINEX
            crnx.rnx2crnx1();
            // dump to file
            let rnx_b_path = format!("test-{}", crnx_name);
            assert_eq!(crnx.to_file(&rnx_b_path).is_ok(), true);
            // run testbench
            run_comparison(
                &format!("../test_resources/CRNX/V1/{}", crnx_name),
                &rnx_b_path);
            //let _ = std::fs::remove_file(&rnx_b_path);
        }
    }
}
