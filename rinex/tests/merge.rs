#[cfg(test)]
mod merge {
    use rinex::Rinex;
    #[test]
    fn test_merge_type_mismatch() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path1 = test_resources.to_owned() + "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut r1 = Rinex::from_file(&path1).unwrap();
        let path2 = test_resources.to_owned() + "OBS/V3/LARM0630.22O";
        let r2 = Rinex::from_file(&path2).unwrap();
        assert_eq!(r1.merge(&r2).is_err(), true)
    }
    /*#[test]
    /// Tests `Merge()` ops
    fn test_merge_rev_mismatch() {
        let manifest = env!("CARGO_MANIFEST_DIR");
        let path1 = manifest.to_owned() + "/data/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut r1 = Rinex::from_file(&path1).unwrap();
        let path2 = manifest.to_owned() + "/data/NAV/V2/amel0010.21g";
        let r2 = Rinex::from_file(&path2).unwrap();
        assert_eq!(r1.merge(&r2).is_err(), true)
    }*/
    #[test]
    fn test_merge_basic() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path1 = test_resources.to_owned() + "NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut r1 = Rinex::from_file(&path1).unwrap();
        let path2 = test_resources.to_owned() + "NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx";
        let r2 = Rinex::from_file(&path2).unwrap();
        assert_eq!(r1.merge(&r2).is_ok(), true)
        //println!("is merged          : {}", rinex.is_merged_rinex());
        //println!("boundaries: \n{:#?}", rinex.merge_boundaries());
    }
}
