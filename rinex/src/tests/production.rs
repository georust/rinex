#[cfg(test)]
mod test {
    use crate::tests::toolkit::{compare_with_panic, random_name};
    use crate::*;
    use std::path::Path;
    fn testbench(path: &str) {
        // parse this file
        let rnx = Rinex::from_file(path).unwrap(); // already tested elsewhere
        let tmp_path = format!("test-{}.rnx", random_name(5));
        assert_eq!(rnx.to_file(&tmp_path).is_ok(), true); // test writer
        let copy = Rinex::from_file(&tmp_path);
        assert_eq!(copy.is_ok(), true); // content should be valid
        let copy = copy.unwrap();
        // run comparison
        if copy != rnx {
            compare_with_panic(&copy, &rnx, path);
        }
        println!("production test passed for \"{}\"", path);
        // remove copy
        let _ = std::fs::remove_file(tmp_path);
    }
    #[test]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn obs_v2() {
        let prefix = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2");
        // does not work well on very old rinex like V2/KOSG..
        for file in vec![
            "AJAC3550.21O",
            "aopr0010.17o",
            "barq071q.19o",
            "delf0010.21o",
            "npaz3550.21o",
            "rovn0010.21o",
            "wsra0010.21o",
            "zegv0010.21o",
        ] {
            let path = prefix.to_path_buf().join(file);

            let fullpath = path.to_string_lossy();
            testbench(&fullpath.to_string());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn obs_v3() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_v4() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn clocks_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/CLK/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn nav_v2() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V2/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    #[ignore]
    fn nav_v3() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V3/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }
    /*
    #[test]
    #[cfg(feature = "flate2")]
    fn nav_v4() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/NAV/V4/";
        for file in std::fs::read_dir(folder).unwrap() {
            let fp = file.unwrap();
            let fp = fp.path();
            testbench(fp.to_str().unwrap());
        }
    }*/
}
