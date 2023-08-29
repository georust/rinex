#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;
    use crate::tests::toolkit::*;
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
            
            let crnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("CRNX/V1")
                .join(crnx_name);

            let rnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("OBS/V2")
                .join(rnx_name);

            let rnx = Rinex::from_file(&rnx_path.to_string_lossy());
            assert!(
                rnx.is_ok(),
                "failed to parse test file \"{}\"",
                rnx_path.to_string_lossy()
            );
            let rnx = rnx.unwrap();

            // convert to CRINEX1
            println!("compressing \"{}\"..", rnx_path.to_string_lossy());
            let dut = rnx.rnx2crnx1();

            // parse model
            let model = Rinex::from_file(&crnx_path.to_string_lossy());
            assert!(
                model.is_ok(),
                "failed to parse test file \"{}\"",
                crnx_path.to_string_lossy()
            );

            // compare to CRINEX1 model
            let model = model.unwrap();
            compare_with_panic(
                &dut,
                &model,
                &format!("compression::crinx1::{}", rnx_path.to_string_lossy()),
            );
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

            let rnx = Rinex::from_file(&rnx_path);
            assert!(
                rnx.is_ok(),
                "Failed to parse test pool file \"{}\"",
                testfile
            );

            // compress
            let rnx = rnx.unwrap();
            let compressed = rnx.rnx2crnx1();

            assert!(
                compressed.to_file("test.crx").is_ok(),
                "{}{}",
                "failed to format compressed rinex",
                testfile
            );

            // test reciprocity
            let uncompressed = compressed.crnx2rnx();
            assert!(
                rnx == uncompressed,
                "{}{}",
                "reciprocity test failed for \"{}\"",
                testfile
            );

            // remove generated file
            let _ = std::fs::remove_file("test.crx");
        }
    }
}
