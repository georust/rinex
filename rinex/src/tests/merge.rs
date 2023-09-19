#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::Merge;
    use std::path::PathBuf;
    #[test]
    fn fail_on_type_mismatch() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");
        let path1 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");
        let path2 = test_resources
            .clone()
            .join("OBS")
            .join("V3")
            .join("LARM0630.22O");
        let mut r1 = Rinex::from_file(&path1.to_string_lossy()).unwrap();
        let r2 = Rinex::from_file(&path2.to_string_lossy()).unwrap();
        assert_eq!(r1.merge_mut(&r2).is_err(), true)
    }
    #[test]
    fn merge() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");
        let path1 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");
        let rnx_a = Rinex::from_file(&path1.to_string_lossy());
        assert!(
            rnx_a.is_ok(),
            "failed to parse NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
        );
        let path2 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("CBW100NLD_R_20210010000_01D_MN.rnx");
        let rnx_b = Rinex::from_file(&path2.to_string_lossy());
        assert!(
            rnx_b.is_ok(),
            "failed to parse NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx"
        );

        let rnx_a = rnx_a.unwrap();
        let rnx_b = rnx_b.unwrap();
        let merged = rnx_a.merge(&rnx_b);
        assert!(
            merged.is_ok(),
            "failed to merge NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx into NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
        );

        // dump
        let merged = merged.unwrap();
        assert!(
            merged.to_file("merge.txt").is_ok(),
            "failed to generate file previously merged"
        );
        assert!(
            merged.is_merged(),
            "is_merged() should be true after merging!"
        );

        // parse back
        let rnx = Rinex::from_file("merge.txt");
        assert!(rnx.is_ok(), "Failed to parsed back previously merged file");

        let rnx = rnx.unwrap();
        assert!(
            rnx.is_merged(),
            "failed to identify a merged file correctly"
        );

        /*
         * Unlock reciprocity test in near future
         *  NAV file production does not work correctly at the moment,
         *  due to formatting issues
         */
        // assert_eq!(rnx, merged, "Merge::ops reciprocity");

        // remove file we just generated
        let _ = std::fs::remove_file("merge.txt");
    }
}
