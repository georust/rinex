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
    fn crinex1() {
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
            let crnx_path = format!("../test_resources/CRNX/V1/{}", crnx_name);
            let rnx_path = format!("../test_resources/OBS/V2/{}", rnx_name);
            
            let rnx = Rinex::from_file(&rnx_path)
                .expect("failed to parse test pool file");
            
            // convert to CRINEX1 
            println!("compressing \"{}\"..", rnx_path);
            let dut = rnx.rnx2crnx1(); 
            
            // parse model
            let model = Rinex::from_file(&crnx_path)
                .expect("failed to parse test pool file");
            
            // compare to CRINEX1 model
            test_toolkit::compare_with_panic(&dut, &model, &format!("compression::crinx1::{}", rnx_path));
        }
    }
    //#[test]
    fn crinex1_reciprocity() {
        let pool = vec![
            ("AJAC3550.21O"),
            ("aopr0010.17o"),
            ("npaz3550.21o"),
            ("pdel0010.21o"),
            ("wsra0010.21o"),
            ("zegv0010.21o"),
        ];
        for testfile in pool {
            let rnx_path = format!("../test_resources/OBS/V2/{}", testfile);
            
            let rnx = Rinex::from_file(&rnx_path)
                .expect("failed to parse test pool file");
            
            // compress
            let compressed = rnx.rnx2crnx1(); 

            assert!(compressed.to_file("test.crx").is_ok(),
                "{}{}", 
                "failed to format compressed rinex", testfile);

            // test reciprocity
            let uncompressed = compressed.crnx2rnx();
            assert!(rnx == uncompressed, 
                "{}{}", 
                "reciprocity test failed for \"{}\"", testfile);
        }
    }
}
