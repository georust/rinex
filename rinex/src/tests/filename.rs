use crate::prelude::*;
use std::path::Path;

// Test our standardized name generator does follow the specs
#[test]
fn short_filename_conventions() {
    for (testfile, expected) in [
        //FIXME: slightly wrong due to HIFITIME PB @ DOY(GNSS)
        ("OBS/V2/AJAC3550.21O", "AJAC3550.21O"),
        ("OBS/V2/rovn0010.21o", "ROVN0020.20O"),
        // FIXME on next hifitime release
        ("OBS/V3/LARM0010.22O", "LARM0010.21O"),
        // FIXME on next hifitime release
        ("OBS/V3/pdel0010.21o", "PDEL0020.20O"),
        // FIXME on next hifitime release
        ("CRNX/V1/delf0010.21d", "DELF0020.20D"),
        // FIXME on next hifitime release
        ("CRNX/V1/zegv0010.21d", "ZEGV0020.20D"),
        // FIXME on next hifitime release
        ("CRNX/V3/DUTH0630.22D", "DUTH0630.22D"),
        // FIXME on next hifitime release
        ("CRNX/V3/VLNS0010.22D", "VLNS0010.21D"),
        ("MET/V2/abvi0010.15m", "ABVI0010.15M"),
        // FIXME on next hifitime release
        ("MET/V2/clar0020.00m", "CLAR0020.00M"),
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join(testfile);

        let rinex = Rinex::from_file(fp.to_string_lossy().as_ref()).unwrap();

        let _actual_filename = fp.file_name().unwrap().to_string_lossy().to_string();

        let output = rinex.standard_filename(true, None, None); // force short

        assert_eq!(output, expected, "bad short filename generated");
    }
}

// Test our standardized name generator does follow the specs
#[test]
fn long_filename_conventions() {
    for (testfile, expected, custom_suffix) in [
        (
            "OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            "ACOR00ESP_R_20213552359_01D_30S_MO.rnx", //FIXME: hifitime DOY(GNSS)
            None,
        ),
        (
            "OBS/V3/ALAC00ESP_R_20220090000_01D_30S_MO.rnx",
            "ALAC00ESP_R_20220092359_01D_13M_MO.rnx", //FIXME: hifitime DOY(GNSS)
            None,
        ),
        (
            "CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            "ESBC00DNK_R_20201772359_01D_30S_MO.crx.gz", //FIXME: hifitime DOY(GNSS)
            Some(".gz"),
        ),
    ] {
        let fp = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join(testfile);

        let rinex = Rinex::from_file(fp.to_string_lossy().as_ref()).unwrap();

        //FIXME: hifitime DOY(GNSS) : use filename directly
        let _standard_filename = fp.file_name().unwrap().to_string_lossy().to_string();

        let output = rinex.standard_filename(false, custom_suffix, None);
        assert_eq!(output, expected, "bad filename generated");
    }
}
