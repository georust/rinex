#[cfg(test)]
mod test {
    use rinex::*;
    use std::str::FromStr;
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
    fn test_parser() {
        let test_resources = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let test_data = vec![
			//"ATX",
			//"CLK",
			//"CRNX",
			//"MET",
			"NAV",
			//"OBS",
		];
        for data in test_data {
            let data_path = std::path::PathBuf::from(
                test_resources.to_owned() + data
            );
            for revision in std::fs::read_dir(data_path)
                .unwrap() {
                let rev = revision.unwrap();
                let rev_path = rev.path();
                let rev_fullpath = &rev_path.to_str().unwrap(); 
                for entry in std::fs::read_dir(rev_fullpath)
                    .unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    let full_path = &path.to_str().unwrap();
                    let is_hidden = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .starts_with(".");
                    if is_hidden {
                        continue // not a test resource
                    }
                    let is_generated_file = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with("-copy");
                    if is_generated_file {
                        continue // not a test resource
                    }
                    let mut is_gzip_encoded = entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with(".gz");
                    is_gzip_encoded |= entry
                        .file_name()
                        .to_str()
                        .unwrap()
                        .ends_with(".Z");
                    if is_gzip_encoded && !cfg!(feature="with-gzip") {
                        continue // do not run in this build configuration
                    }
                    // PARSER
                    println!("Parsing file: \"{}\"", full_path);
                    let rinex = Rinex::from_file(full_path);
                    assert_eq!(rinex.is_ok(), true);
                    // HEADER
                    let rinex = rinex.unwrap();
                    // RECORD
                    match data {
                        "ATX" => { // ATX record
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_navigation_rinex(), false);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            assert_eq!(rinex.is_antex_rinex(), true);
                        },
                        "NAV" => {
                            // NAV files checks
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_navigation_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            let record = rinex.record.as_nav().unwrap();
                            let mut epochs = record.keys();
                            // Testing event description finder
                            if let Some(event) = epochs.nth(0) {
                                // [!] with dummy t0 = 1st epoch timestamp
                                //     this will actually return `header section` timestamps
                            }
                        },
                        "OBS" => {
                            // OBS files checks
                            let obs = rinex.header.obs.as_ref();
                            assert_eq!(obs.is_some(), true);
                            assert_eq!(rinex.is_observation_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            let obs = obs.unwrap();
                            if obs.clock_offset_applied {
                                // epochs should always have a RCVR clock offset
                                // test that with iterator
                            }
                            let record = rinex.record
                                .as_obs()
                                .unwrap();
                            let mut epochs = record.keys();
                            // Testing event description finder
                            if let Some(event) = epochs.nth(0) {
                                // [!] with dummy t0 = 1st epoch timestamp
                                //     this will actually return `header section` timestamps
                            }
                        },
                        "CRNX" => {
                            // compressed OBS files checks
                            assert_eq!(rinex.header.obs.is_some(), true);
                            assert_eq!(rinex.is_observation_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            let record = rinex.record.as_obs().unwrap();
                            let mut epochs = record.keys();
                            // Testing event description finder
                            if let Some(event) = epochs.nth(0) {
                                // [!] with dummy t0 = 1st epoch timestamp
                                //     this will actually return `header section` timestamps
                            }
                        },
                        "MET" => {
                            // METEO files checks
                            assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.is_meteo_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_some(), true);
                            assert_eq!(rinex.header.obs.is_none(), true);
                            let record = rinex.record.as_meteo().unwrap();
                            let mut epochs = record.keys();
                            // Testing event description finder
                            if let Some(event) = epochs.nth(0) {
                                // [!] with dummy t0 = 1st epoch timestamp
                                //     this will actually return `header section` timestamps
                            }
                        },
                        "CLK" => {
                            assert_eq!(rinex.is_clocks_rinex(), true);
                            assert_eq!(rinex.header.meteo.is_none(), true);
                            //assert_eq!(rinex.header.obs.is_none(), true);
                            assert_eq!(rinex.header.clocks.is_some(), true);
                        },
                        _ => {}
                    }
                    /*
                     * // SPECIAL METHODS
                    println!("sampling interval  : {:#?}", rinex.sampling_interval());
                    println!("sampling dead time : {:#?}", rinex.dead_times());
                    println!("abnormal epochs    : {:#?}", rinex.epoch_anomalies(None));
                    // COMMENTS
                    println!("---------- Header Comments ----- \n{:#?}", rinex.header.comments);
                    println!("---------- Body   Comments ------- \n{:#?}", rinex.comments);
                    // MERGED RINEX special ops
                    println!("---------- Merged RINEX special ops -----------\n");
                    println!("is merged          : {}", rinex.is_merged());
                    println!("boundaries: \n{:#?}", rinex.merge_boundaries());
                    // Test RINEX writer 
                    rinex.to_file("output").unwrap();
                    // suppress 
                    let _ = std::fs::remove_file("output");
                    //TODO test bench
                    //let identical = diff_is_strictly_identical("test", "data/MET/V2/abvi0010.15m").unwrap();
                    //assert_eq!(identical, true) */
                }
            }
        }
    }
}
