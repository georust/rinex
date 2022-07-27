#[cfg(test)]
mod sampling {
    use rinex::*;
    #[test]
    fn test_decimate_mut_nav_by_interval() {
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
        rinex.decimate_by_interval_mut(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 09:45:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), 4);

        let interval = std::time::Duration::from_secs(5*3600);
        rinex.decimate_by_interval_mut(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), 3); 
    }
    #[test]
    fn test_decimate_nav_by_interval() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&path).unwrap();
        let epochs = rinex.epochs();
        let origin = epochs.len();
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
        let rinex = rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 00:15:00
        // 05:00:00
        // 09:45:00
        // 10:10:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), origin); // interval too small: nothing changed

        let interval = std::time::Duration::from_secs(3600); 
        let rinex = rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 09:45:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), 4);
        
        let interval = std::time::Duration::from_secs(5 * 3600); 
        let rinex = rinex.decimate_by_interval(interval);
        let epochs = rinex.epochs();
        ////////////////////////
        // Epochs in this file:
        // 00:00:00
        // 05:00:00
        // 15:40:00
        ////////////////////////
        assert_eq!(epochs.len(), 3);
    }
    #[test]
    fn test_decimate_obs_by_interval_mut() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/zegv0010.21o";
        let rinex = Rinex::from_file(&path).unwrap();
        ////////////////////////
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 00 30.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 01 30.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 02 30.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 03 30.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 04 30.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 05 30.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 06 30.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 07 30.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 08 30.0000000
        // 21 01 01 00 09 00.0000000
        ////////////////////////
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 19);

        let interval = std::time::Duration::from_secs(60);
        let rinex = rinex.decimate_by_interval(interval);
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 09 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 10);
        
        let interval = std::time::Duration::from_secs(120);
        let rinex = rinex.decimate_by_interval(interval);
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 08 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 5);
    }
    #[test]
    fn test_decimate_obs_by_ratio() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/zegv0010.21o";
        let rinex = Rinex::from_file(&path).unwrap();
        ////////////////////////
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 00 30.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 01 30.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 02 30.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 03 30.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 04 30.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 05 30.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 06 30.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 07 30.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 08 30.0000000
        // 21 01 01 00 09 00.0000000
        ////////////////////////
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 19);

        let rinex = rinex.decimate_by_ratio(2);
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 09 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 10);
        
        let rinex = rinex.decimate_by_ratio(3);
        // Epochs in this file:
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 09 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 4);
    }
    #[test]
    fn test_decimate_obs_by_ratio_mut() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/zegv0010.21o";
        let mut rinex = Rinex::from_file(&path).unwrap();
        ////////////////////////
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 00 30.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 01 30.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 02 30.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 03 30.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 04 30.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 05 30.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 06 30.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 07 30.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 08 30.0000000
        // 21 01 01 00 09 00.0000000
        ////////////////////////
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 19);

        rinex.decimate_by_ratio_mut(2);
        // Epochs in this file:
        // 21 01 01 00 00 00.0000000
        // 21 01 01 00 01 00.0000000
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 03 00.0000000
        // 21 01 01 00 04 00.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 06 00.0000000
        // 21 01 01 00 07 00.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 09 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 10);
        
        rinex.decimate_by_ratio_mut(3);
        // Epochs in this file:
        // 21 01 01 00 02 00.0000000
        // 21 01 01 00 05 00.0000000
        // 21 01 01 00 08 00.0000000
        // 21 01 01 00 09 00.0000000
        let epochs = rinex.epochs();
        assert_eq!(epochs.len(), 4);
    }
/* is this a rounding issue? ...
    #[test]
    fn test_average_epoch_duration() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/zegv0010.21o";
        let rinex = Rinex::from_file(&path).unwrap();
        println!("{:#?}", rinex.epochs());
        println!("{:#?}", rinex.epochs().len());
        ////////////////////////
        // 21 01 01 00 08 00.0000000 
        // 21 01 01 00 08 30.0000000 
        // 21 01 01 00 09 00.0000000
        ////////////////////////
        let header = &rinex.header;
        assert_eq!(header.sampling_interval.is_some(), true);
        let interval = header.sampling_interval.unwrap();
        let expected = std::time::Duration::from_secs(interval as u64);
        assert_eq!(rinex.average_epoch_duration(), expected);
    }
*/
}
