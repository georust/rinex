#[cfg(test)]
mod sampling {
    use rinex::*;
    #[test]
    fn test_decimate_by_interval() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();
        let epochs = rinex.epochs();
        let origin_len = epochs.len(); 
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 00:15:00
        // 05:00:00
        // 09:45:00
        // 10:10:00
        // 15:40:00
        ////////////////////////
        rinex.decimate_by_interval(std::time::Duration::from_secs(1));
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), origin_len); // interval too small: nothing changed

        rinex.decimate_by_interval(std::time::Duration::from_secs(3600)); // removes 2nd epoch
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), origin_len -1);
    }
}
