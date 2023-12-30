use crate::prelude::*;
use std::path::{Path, PathBuf};

// Test our standardized name generator does follow the specs
#[test]
fn short_filename_conventions() {
    for (testfile, expected, lowercase, batch_num) in [
        //FIXME: slightly wrong due to HIFITIME PB @ DOY(GNSS)
        ("OBS/V2/AJAC3550.21O", "AJAC3540.21O", false, None),
        ("OBS/V2/rovn0010.21o", "rovn0010.20o", true, None),
        // FIXME on next hifitime release
        ("OBS/V3/LARM0010.22O", "LARM0010.21O", false, None),
        // FIXME on next hifitime release
        ("OBS/V3/pdel0010.21o", "PDEL0010.20O", false, None),
        // FIXME on next hifitime release
        ("CRNX/V1/delf0010.21d", "delf0010.20d", true, None),
        // FIXME on next hifitime release
        ("CRNX/V1/zegv0010.21d", "ZEGV0010.20D", false, None),
        // FIXME on next hifitime release
        ("CRNX/V3/DUTH0630.22D", "DUTH0620.22D", false, None),
        // FIXME on next hifitime release
        ("CRNX/V3/VLNS0010.22D", "VLNS0010.21D", false, None),
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join(testfile);

        let rinex = Rinex::from_file(&fp.to_string_lossy().to_string()).unwrap();

        let standard_filename = fp.file_name().unwrap().to_string_lossy().to_string();

        let output = rinex
            .standardized_short_filename(lowercase, batch_num, None)
            .unwrap();

        assert_eq!(output, expected, "bad short filename generated");
    }
}

// Test our standardized name generator does follow the specs
#[test]
fn long_filename_conventions() {
    for (testfile, expected) in [
        //FIXME: hifitime DOY(GNSS)
        //       remove expected completely, use PathBuf.file_name() directly
        (
            "OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            "ACOR00XXX_R_20213542359_01D_30S_MO.rnx",
        ),
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join(testfile);

        let rinex = Rinex::from_file(&fp.to_string_lossy().to_string()).unwrap();

        let standard_filename = fp.file_name().unwrap().to_string_lossy().to_string();

        let output = rinex.standardized_filename(None).unwrap();

        assert_eq!(output, expected, "bad filename generated");
    }
}
