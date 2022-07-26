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
        let interval = std::time::Duration::from_secs(1);
        rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), origin_len); // interval too small: nothing changed

        let interval = std::time::Duration::from_secs(3600); 
        // will drop 00:00:00->00:15:00
        // will drop 09:45:00->10:10:00
        rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 09:45:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), origin_len -2);

        let interval = std::time::Duration::from_secs(5*3600);
        rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), 3); 
    }
}
