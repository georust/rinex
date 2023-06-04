#[cfg(test)]
mod merge {
    use rinex::prelude::*;
    use rinex::Merge;
    #[test]
    fn merge_fail_on_type_mismatch() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path1 = test_resources.to_owned() + "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut r1 = Rinex::from_file(&path1).unwrap();
        let path2 = test_resources.to_owned() + "OBS/V3/LARM0630.22O";
        let r2 = Rinex::from_file(&path2).unwrap();
        assert_eq!(r1.merge_mut(&r2).is_err(), true)
    }
    #[test]
    fn merge() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path1 = test_resources.to_owned() + "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rnx_a = Rinex::from_file(&path1);
        assert!(rnx_a.is_ok(), "Failed to parse NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx test resource");
        let path2 = test_resources.to_owned() + "NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx";
        let rnx_b = Rinex::from_file(&path2);
        assert!(rnx_b.is_ok(), "Failed to parse NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx test resource");
        
        let rnx_a = rnx_a.unwrap();
        let rnx_b = rnx_b.unwrap();
        let merged = rnx_a.merge(&rnx_b);
        assert!(merged.is_ok(), "Failed to merge NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx into NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx");

        // dump
        let merged = merged.unwrap();
        assert!(merged.to_file("merge.txt").is_ok(), "Failed to generate Merged file");
        
        // parse back
        let rnx = Rinex::from_file("merge.txt");
        assert!(rnx.is_ok(), "Failed to parsed back previously merged file");
        
        //TODO
        // this does not work, due to NAV RINEX formatting issues,
        // unlock this in near future
        // assert_eq!(rnx, merged, "Merge::ops reciprocity");

        let _ = std::fs::remove_file("merge.txt"); // remove file we just generated
    }
}
