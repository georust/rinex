#[cfg(test)]
mod test {
    use rinex::*;
    //use std::str::FromStr;
    use std::process::Command;
    /// Runs `diff` to determines whether f1 & f2 
    /// are strictly identical or not
    fn diff_is_strictly_identical (f1: &str, f2: &str) -> Result<bool, std::string::FromUtf8Error> {
        let output = Command::new("diff")
            .arg("-q")
            .arg("-Z")
            .arg(f1)
            .arg(f2)
            .output()
            .expect("failed to execute \"diff\"");
        let output = String::from_utf8(output.stdout)?;
        Ok(output.len()==0)
    }
    #[test]
    fn test_obs_v2_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/";
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
            // remove copy to not disturb other tests browser
            let _ = std::fs::remove_file(copy_path);
        }
    }
    #[test]
    fn test_obs_v3_production() {
        let folder = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V3/";
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
            // remove copy to not disturb other tests browser
            let _ = std::fs::remove_file(copy_path);
        }
    }
}
