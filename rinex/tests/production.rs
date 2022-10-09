#[cfg(test)]
mod test {
    use rinex::*;
    //use std::str::FromStr;
    #[test]
    fn obs_v2_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rnx_a = Rinex::from_file(fp)
				.unwrap(); // already tested elsewhere
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
			assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
			let rnx_b = Rinex::from_file(&copy_path);
			assert_eq!(rnx_b.is_ok(), true);
			let rnx_b = rnx_b
				.unwrap();
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn obs_v3_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rnx_a = Rinex::from_file(fp)
				.unwrap(); // already tested elsewhere
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
			assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
			let rnx_b = Rinex::from_file(&copy_path);
			assert_eq!(rnx_b.is_ok(), true);
			let rnx_b = rnx_b
				.unwrap();
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn meteo_v2_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rnx_a = Rinex::from_file(fp)
				.unwrap(); // already tested elsewhere
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
			assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
			let rnx_b = Rinex::from_file(&copy_path);
			assert_eq!(rnx_b.is_ok(), true);
			let rnx_b = rnx_b
				.unwrap();
			//assert_eq!(rnx_a, rnx_b);
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn meteo_v4_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rnx_a = Rinex::from_file(fp)
				.unwrap(); // already tested elsewhere
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
			assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
			let rnx_b = Rinex::from_file(&copy_path);
			assert_eq!(rnx_b.is_ok(), true);
			let rnx_b = rnx_b
				.unwrap();
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn nav_v2_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rnx_a = Rinex::from_file(fp)
				.unwrap(); // already tested elsewhere
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
			assert_eq!(rnx_a.to_file(&copy_path).is_ok(), true);
			let rnx_b = Rinex::from_file(&copy_path);
			assert_eq!(rnx_b.is_ok(), true);
			let rnx_b = rnx_b
				.unwrap();
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
/*
    #[test]
    fn nav_v3_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rinex = Rinex::from_file(fp);
            assert_eq!(rinex.is_ok(), true);
            let rinex = rinex.unwrap();
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
            assert_eq!(rinex.to_file(&copy_path).is_ok(), true);
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn nav_v4_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            let fp = fp.to_str().unwrap();
            // parse this rinex
            let rinex = Rinex::from_file(fp);
            assert_eq!(rinex.is_ok(), true);
            let rinex = rinex.unwrap();
            // produce a copy
            let copy_path = fp.to_owned() + "-copy";
            assert_eq!(rinex.to_file(&copy_path).is_ok(), true);
            // remove copy not to disturb other test browsers
            let _ = std::fs::remove_file(copy_path);
        }
    }
*/
}
