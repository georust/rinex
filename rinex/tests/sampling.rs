#[cfg(test)]
mod sampling {
    use rinex::prelude::*;
    use rinex::preprocessing::*;
    use std::collections::HashMap;
    #[test]
    fn epoch_intervals() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&path).unwrap();
        let histogram = rinex.epoch_intervals();
        let expected: HashMap<_, _> = [
            (Duration::from_seconds(15.0 * 60.0), 1),
            (Duration::from_seconds(4.0 * 3600.0 + 45.0 * 60.0), 2),
            (Duration::from_seconds(25.0 * 60.0), 1),
            (Duration::from_seconds(5.0 * 3600.0 + 30.0 * 60.0), 1),
        ]
        .into_iter()
        .collect();

        assert_eq!(histogram, expected);
        assert_eq!(histogram.len(), expected.len());
    }
    #[test]
    fn decimate_by_interval() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();
        let initial_epochs = rinex.epochs();
        rinex.decimate_by_interval_mut(Duration::from_seconds(10.0));
        assert_eq!(rinex.epochs(), initial_epochs); // unchanged, interval too small for this file
        rinex.decimate_by_interval_mut(Duration::from_hours(1.0));
        assert_eq!(rinex.epochs().len(), initial_epochs.len() - 2);
    }
    #[test]
    fn decimate_by_ratio() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();

        rinex.decimate_by_ratio_mut(2);
        assert_eq!(rinex.epochs().len(), 3);

        rinex.decimate_by_ratio_mut(2);
        assert_eq!(rinex.epochs().len(), 2);
    }
}
