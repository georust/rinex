#[cfg(test)]
mod sampling {
    use rinex::*;
    use std::str::FromStr;
    use std::process::Command;
    /// Tests record `Decimate()` ops 
    fn test_record_decimation() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();
        let original : Vec<&epoch::Epoch> = rinex.record.as_nav().unwrap().keys().collect();
        println!("LEN {}", original.len());
        rinex.resample(std::time::Duration::from_secs(1));
        rinex.resample(std::time::Duration::from_secs(10*60));
    }
}
