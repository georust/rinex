#[cfg(test)]
mod test {
    use rinex::*;
    //use std::str::FromStr;
    use std::process::Command;
    fn run_diffz (path1: &str, path2: &str) -> Result<String, std::string::FromUtf8Error> {
        let output = Command::new("diff")
            .arg("-Z")
            .arg(path1)
            .arg(path2)
            .output()
            .expect("failed to execute \"diff -z\"");
        let output = String::from_utf8(output.stdout)?;
        Ok(output)
    }
    fn observation_report_on_failure (rnx_a: &Rinex, rnx_b: &Rinex) {
        let rec_a = rnx_a.record.as_obs().unwrap();
        let rec_b = rnx_b.record.as_obs().unwrap();
        for (e_a, (clk_offset_a, vehicules_a)) in rec_a.iter() {
            println!("\nInvestigating {:?}", e_a);
            let (clk_offset_b, vehicules_b) = rec_b.get(e_a)
                .unwrap(); // causes a panic that will help debug
            assert_eq!(clk_offset_a, clk_offset_b);
            for (sv_a, observables_a) in vehicules_a.iter() {
                println!("Testing {:?}", sv_a);
                let observables_b = vehicules_b.get(sv_a)
                    .unwrap(); // helpful panic
                assert_eq!(observables_a, observables_b);
                for (code_a, obs_a) in observables_a {
                    println!("Testing {:?}", code_a);
                    let obs_b = observables_b
                        .get(code_a)
                        .unwrap(); // helpful panic
                    println!("Testing {:?}", obs_a);
                    assert_eq!(obs_a, obs_b);
                }
            }
        }

        // reaching this point
        //  means rnx_b has more data than expected
        println!("******* Got more data than expected ********");
        for (e_b, (clk_offset_b, vehicules_b)) in rec_b.iter() {
            println!("\nInvestigating {:?}", e_b);
            let (clk_offset_a, vehicules_a) = rec_a.get(e_b)
                .unwrap();
            assert_eq!(clk_offset_a, clk_offset_b);
            for (sv_b, observables_b) in vehicules_b.iter() {
                println!("Testing {:?}", sv_b);
                let observables_a = vehicules_a.get(sv_b)
                    .unwrap();
                assert_eq!(observables_a, observables_b);
                for (code_b, obs_b) in observables_b {
                    println!("Testing {:?}", code_b);
                    let obs_a = observables_a.get(code_b)
                        .unwrap();
                    println!("Testing {:?}", obs_a);
                    assert_eq!(obs_a, obs_b);
                }
            }
        }
    }
    fn report_on_failure(rnx_a: &Rinex, rnx_b: &Rinex) {
        //TODO
        //not for certain RINEX types
        let epochs_a = rnx_a.epochs();
        let epochs_b = rnx_b.epochs();
        assert_eq!(epochs_a, epochs_b);
        // dig further
        if rnx_a.is_observation_rinex() {
            observation_report_on_failure(&rnx_a, &rnx_b);
        }
        // other types of investigation should be coded here
    }
    fn testbench (path: &str) {
        println!("Running testbench on: \"{}\"", path);
        let rnx_a = Rinex::from_file(path)
            .unwrap(); // tested in parser dedicated testsuite
        // generate a copy 
        let copy_path = path.to_owned() + "-copy";
        assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
        // parse copy
        let rnx_b = Rinex::from_file(&copy_path);
        assert_eq!(rnx_b.is_ok(), true);
        let rnx_b = rnx_b
            .unwrap();
        if rnx_a != rnx_b {
            let diff = run_diffz(path, &copy_path)
                .unwrap();
            println!("******* DIFF ******** \n{}\n *************** ", diff);
            report_on_failure(&rnx_a, &rnx_b);
            panic!("******* TESTBENCH FAILED **********");
        }
        // remove copy not to disturb other test browsers
        // let _ = std::fs::remove_file(copy_path);
    }
    /*#[test]
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
    }*/
    #[test]
    fn meteo_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    /*#[test]
    fn meteo_v4() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/";
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
