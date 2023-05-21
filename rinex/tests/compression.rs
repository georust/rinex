#[cfg(test)]
mod test {
    use rinex::prelude::*;
    /*
     * compare produced CRINEX to official CRINEX
     */
    fn testbench(produced: &str, official: &str) {
        let f1 = Rinex::from_file(produced);
        assert!(f1.is_ok());
        let f1 = f1.unwrap();
        let f2 = Rinex::from_file(official).unwrap();
        let r1 = f1.record.as_obs();
        assert!(r1.is_some());
        let r1 = r1.unwrap();
        let r2 = f2.record.as_obs().unwrap();
        /*
         * testbench
         */
        assert_eq!(r1.len(), r2.len());
        for (expected_e, (expected_clk, expected_svs)) in r2 {
            let epoch = r1.get(expected_e);
            assert!(epoch.is_some());
            let (clk, svs) = epoch.unwrap();
            assert_eq!(clk, expected_clk);
            for (expected_sv, expected_obss) in expected_svs {
                let sv = svs.get(expected_sv);
                assert!(
                    sv.is_some(),
                    "missing {:?} in epoch {:?}",
                    expected_sv,
                    expected_e
                );
                let obss = sv.unwrap();
                //TODO test observations
            }
        }
    }
    //#[test]
    fn testbench_v1() {
        let pool = vec![
            ("AJAC3550.21D", "AJAC3550.21O"),
            ("aopr0010.17d", "aopr0010.17o"),
            ("npaz3550.21d", "npaz3550.21o"),
            ("pdel0010.21d", "pdel0010.21o"),
            ("wsra0010.21d", "wsra0010.21o"),
            ("zegv0010.21d", "zegv0010.21o"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse RINEX
            let path = format!("../test_resources/OBS/V2/{}", rnx_name);
            println!("compressing \"{}\"..", path);
            let rnx = Rinex::from_file(&path);
            assert_eq!(rnx.is_ok(), true);
            // convert to CRINEX1
            let mut crnx = rnx.unwrap();
            crnx.rnx2crnx1();
            // dump
            let path = format!("test.crx");
            assert!(crnx.to_file("test.crx").is_ok());
            // compare to official
            testbench(
                "test.crx",
                &format!("../test_resources/CRNX/V1/{}", crnx_name),
            );
        }
    }
}
