#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::tests::toolkit::random_name;
    use std::{fs::remove_file as fs_remove_file, path::PathBuf};
    #[test]
    #[ignore]
    fn crinex1() {
        let pool = vec![
            //("AJAC3550.21D", "AJAC3550.21O"),
            //("aopr0010.17d", "aopr0010.17o"),
            //("npaz3550.21d", "npaz3550.21o"),
            ("wsra0010.21d", "wsra0010.21o"),
            ("zegv0010.21d", "zegv0010.21o"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;

            let crnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("../")
                .join("test_resources")
                .join("CRNX")
                .join("V1")
                .join(crnx_name);

            let rnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("../")
                .join("test_resources")
                .join("OBS")
                .join("V2")
                .join(rnx_name);

            let fullpath = rnx_path.to_string_lossy().to_string();
            let rnx = Rinex::from_file(&fullpath);
            assert!(
                rnx.is_ok(),
                "failed to parse \"{}\"",
                rnx_path.to_string_lossy()
            );
            let rnx = rnx.unwrap();

            // convert to CRINEX1
            println!("compressing \"{}\"..", rnx_path.to_string_lossy());
            let dut = rnx.rnx2crnx();

            // parse model
            let model_path = crnx_path.to_string_lossy().to_string();
            let model = Rinex::from_file(&model_path);
            assert!(
                model.is_ok(),
                "failed to parse test file \"{}\"",
                crnx_path.to_string_lossy()
            );
        }
    }
    #[test]
    #[ignore]
    fn crinex1_reciprocity() {
        let pool = vec![
            ("AJAC3550.21O"),
            ("aopr0010.17o"),
            ("npaz3550.21o"),
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
            let compressed = rnx.rnx2crnx();

            let tmp_path = format!("test-{}.crx", random_name(8));

            // assert!(
            //     compressed.to_file(&tmp_path).is_ok(),
            //     "{}{}",
            //     "failed to format compressed rinex",
            //     testfile
            // );

            // test reciprocity
            let uncompressed = compressed.crnx2rnx();

            // remove generated file
            let _ = fs_remove_file(&tmp_path);
        }
    }
    #[test]
    #[ignore]
    fn crinex3() {
        let pool = vec![
            (
                "ACOR00ESP_R_20213550000_01D_30S_MO.crx",
                "ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            ),
            ("DUTH0630.22D", "DUTH0630.22O"),
            ("VLNS0010.22D", "VLNS0010.22O"),
            ("VLNS0630.22D", "VLNS0630.22O"),
            ("flrs0010.12d", "flrs0010.12o"),
            ("pdel0010.21d", "pdel0010.21o"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;

            let crnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("../")
                .join("test_resources")
                .join("CRNX")
                .join("V3")
                .join(crnx_name);

            let rnx_path = PathBuf::new()
                .join(env!("CARGO_MANIFEST_DIR"))
                .join("../")
                .join("test_resources")
                .join("OBS")
                .join("V3")
                .join(rnx_name);

            let fullpath = rnx_path.to_string_lossy().to_string();
            let rnx = Rinex::from_file(&fullpath);

            assert!(
                rnx.is_ok(),
                "failed to parse \"{}\"",
                rnx_path.to_string_lossy()
            );
            let rnx = rnx.unwrap();

            // convert to CRINEX3
            println!("compressing \"{}\"..", rnx_path.to_string_lossy());
            let dut = rnx.rnx2crnx();

            // parse model
            let model_path = crnx_path.to_string_lossy().to_string();
            let model = Rinex::from_file(&model_path);

            assert!(
                model.is_ok(),
                "failed to parse test file \"{}\"",
                crnx_path.to_string_lossy()
            );
        }
    }
    #[test]
    #[ignore]
    fn crinex3_reciprocity() {
        let pool = vec![("pdel0010.21o")];
        for testfile in pool {
            let rnx_path = format!("../test_resources/OBS/V3/{}", testfile);

            let rnx = Rinex::from_file(&rnx_path);
            assert!(
                rnx.is_ok(),
                "Failed to parse test pool file \"{}\"",
                testfile
            );

            // compress
            let rnx = rnx.unwrap();
            let compressed = rnx.rnx2crnx();

            let tmp_path = format!("test-{}.crx", random_name(8));

            // assert!(
            //     compressed.to_file(&tmp_path).is_ok(),
            //     "{}{}",
            //     "failed to format compressed rinex",
            //     testfile
            // );

            // test reciprocity
            let uncompressed = compressed.crnx2rnx();

            // remove generated file
            let _ = fs_remove_file(&tmp_path);
        }
    }
}
