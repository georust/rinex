#[cfg(test)]
mod sampling {
    use itertools::Itertools;
    use rinex::prelude::*;
    use rinex::preprocessing::*;
    use std::collections::HashMap;
    use std::str::FromStr;
    #[test]
    fn sampling_histogram() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let rinex = Rinex::from_file(&path).unwrap();
        let histogram = rinex.sampling_histogram();
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
            + "../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();
        let initial_len = rinex.epoch().count();
        rinex.decimate_by_interval_mut(Duration::from_seconds(10.0));
        assert!(
            initial_len == rinex.epoch().count(),
            "decim with too small time interval failed"
        );
        rinex.decimate_by_interval_mut(Duration::from_hours(1.0));
        assert!(
            initial_len == rinex.epoch().count() + 2,
            "failed to decimate to 1 hour epoch interval"
        );
    }
    #[test]
    fn decimate_by_ratio() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned()
            + "../test_resources/NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx";
        let mut rinex = Rinex::from_file(&path).unwrap();

        rinex.decimate_by_ratio_mut(2);
        assert_eq!(rinex.epoch().count(), 3, "decim by 2 failed");

        rinex.decimate_by_ratio_mut(2);
        assert!(rinex.epoch().count() == 2, "decim by 3 + 2 failed");
    }
    #[test]
    fn dominant_sample_rate() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "../test_resources/MET/V2/abvi0010.15m";
        let rinex = Rinex::from_file(&path).unwrap();
        assert_eq!(
            rinex.dominant_sample_rate(),
            Some(Duration::from_seconds(60.0)),
        );
    }
    #[test]
    fn data_gaps() {
        let path = env!("CARGO_MANIFEST_DIR").to_owned() + "../test_resources/MET/V2/abvi0010.15m";
        let rinex = Rinex::from_file(&path).unwrap();
        assert!(
            rinex.data_gaps(None).sorted().eq(vec![
                (
                    Epoch::from_str("2015-01-01T09:00:00 UTC").unwrap(),
                    Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T19:25:00 UTC").unwrap(),
                    Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T22:55:00 UTC").unwrap(),
                    Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T23:09:00 UTC").unwrap(),
                    Duration::from_seconds(7.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T23:52:00 UTC").unwrap(),
                    Duration::from_seconds(31.0 * 60.0)
                ),
            ]),
            "data_gaps(tol=None) failed",
        );
        assert!(
            rinex
                .data_gaps(Some(Duration::from_seconds(10.0 * 60.0)))
                .sorted()
                .eq(vec![
                    (
                        Epoch::from_str("2015-01-01T09:00:00 UTC").unwrap(),
                        Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)
                    ),
                    (
                        Epoch::from_str("2015-01-01T19:25:00 UTC").unwrap(),
                        Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)
                    ),
                    (
                        Epoch::from_str("2015-01-01T22:55:00 UTC").unwrap(),
                        Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)
                    ),
                    (
                        Epoch::from_str("2015-01-01T23:52:00 UTC").unwrap(),
                        Duration::from_seconds(31.0 * 60.0)
                    ),
                ]),
            "data_gaps(tol=10h) failed",
        );
        let gaps: Vec<_> = rinex
            .data_gaps(Some(Duration::from_seconds(1.0 * 3600.0)))
            .collect();
        assert_eq!(
            gaps,
            vec![
                (
                    Epoch::from_str("2015-01-01T09:00:00 UTC").unwrap(),
                    Duration::from_seconds(8.0 * 3600.0 + 51.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T19:25:00 UTC").unwrap(),
                    Duration::from_seconds(10.0 * 3600.0 + 21.0 * 60.0)
                ),
                (
                    Epoch::from_str("2015-01-01T22:55:00 UTC").unwrap(),
                    Duration::from_seconds(3.0 * 3600.0 + 1.0 * 60.0)
                ),
            ]
        );
    }
}
