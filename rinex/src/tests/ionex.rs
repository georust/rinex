#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::Path;
    #[test]
    #[cfg(feature = "flate2")]
    fn v1_ckmg0090_12i() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("IONEX")
            .join("V1")
            .join("CKMG0090.21I.gz");
        let fullpath = path.to_string_lossy();

        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse IONEX/V1CKMG0090.21I.gz");

        let rinex = rinex.unwrap();
        assert_eq!(
            rinex.tec_fixed_altitude(),
            Some(350.0),
            "bad fixed altitude"
        );
        assert_eq!(
            rinex.tec_rms().count(),
            0,
            "falsely identified some RMS maps"
        );
        assert_eq!(
            rinex.epoch().count(),
            25,
            "wrong amount of epochs identified"
        );
        assert_eq!(
            rinex.first_epoch(),
            Some(Epoch::from_gregorian_utc(2021, 1, 9, 0, 0, 0, 0))
        );
        assert_eq!(
            rinex.last_epoch(),
            Some(Epoch::from_gregorian_utc(2021, 1, 10, 0, 0, 0, 0))
        );
        assert_eq!(
            rinex.dominant_sample_rate(),
            Some(Duration::from_hours(1.0)),
            "bad dominant sample rate identified"
        );
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v1_jplg0010_17i() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("IONEX")
            .join("V1")
            .join("jplg0010.17i.gz");
        let fullpath = path.to_string_lossy();

        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse IONEX/jplg0010.17i.gz");

        let rinex = rinex.unwrap();
        assert_eq!(
            rinex.tec_fixed_altitude(),
            Some(450.0),
            "bad fixed altitude"
        );
        assert!(rinex.tec_rms().count() > 0, "failed to identify RMS maps");
        assert!(
            rinex.tec().count() > 0,
            "failed to parse both RMS + TEC maps"
        );
        assert_eq!(
            rinex.tec().count(),
            rinex.tec_rms().count(),
            "this file contains one RMS map per TEC map"
        );

        assert_eq!(
            rinex.dominant_sample_rate(),
            Some(Duration::from_hours(2.0)),
            "bad dominant sample rate identified"
        );
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v1_ckmg0020_22i() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("IONEX")
            .join("V1")
            .join("CKMG0020.22I.gz");
        let fullpath = path.to_string_lossy();

        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse IONEX/V1/CKMG0020.22I.gz");

        let rinex = rinex.unwrap();
        assert!(rinex.is_ionex());
        let header = rinex.header.clone();
        assert_eq!(header.version.major, 1);
        assert_eq!(header.version.minor, 0);
        assert!(header.ionex.is_some());
        let header = header.ionex.as_ref().unwrap();

        let grid = header.grid.clone();
        assert_eq!(grid.height.start, 350.0);
        assert_eq!(grid.height.end, 350.0);
        assert!(rinex.is_ionex_2d());
        assert_eq!(grid.latitude.start, 87.5);
        assert_eq!(grid.latitude.end, -87.5);
        assert_eq!(grid.latitude.spacing, -2.5);
        assert_eq!(grid.longitude.start, -180.0);
        assert_eq!(grid.longitude.end, 180.0);
        assert_eq!(grid.longitude.spacing, 5.0);
        assert_eq!(header.exponent, -1);
        assert_eq!(header.base_radius, 6371.0);
        assert_eq!(header.elevation_cutoff, 0.0);
        assert_eq!(header.mapping, None);

        assert_eq!(
            rinex.tec_fixed_altitude(),
            Some(350.0),
            "bad fixed altitude"
        );
        assert_eq!(
            rinex.tec_rms().count(),
            0,
            "falsely identified some RMS maps"
        );
        assert_eq!(
            rinex.dominant_sample_rate(),
            Some(Duration::from_hours(1.0)),
            "bad dominant sample rate identified"
        );

        // epoch [1]
        // let e = Epoch::from_gregorian_utc(2022, 1, 2, 0, 0, 0, 0);
        // let data = record.get(&e);
        // let (tec, _, _) = data.unwrap();
        // for p in tec {
        //     assert_eq!(p.altitude, 350.0);
        //     if p.latitude == 87.5 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 9.2).abs() < 1E-3);
        //         }
        //         if p.longitude == -175.0 {
        //             assert!((p.value - 9.2).abs() < 1E-3);
        //         }
        //     }
        //     if p.latitude == 85.0 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 9.2).abs() < 1E-3);
        //         }
        //     }
        //     if p.latitude == 32.5 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 17.7).abs() < 1E-3);
        //         }
        //         if p.longitude == -175.0 {
        //             assert!((p.value - 16.7).abs() < 1E-3);
        //         }
        //     }
        // }
        // // epoch [N-2]
        // let e = Epoch::from_gregorian_utc(2022, 1, 2, 23, 0, 0, 0);
        // let data = record.get(&e);
        // let (tec, _, _) = data.unwrap();
        // for p in tec {
        //     assert_eq!(p.altitude, 350.0);
        //     if p.latitude == 87.5 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 9.2).abs() < 1E-3);
        //         }
        //         if p.longitude == -175.0 {
        //             assert!((p.value - 9.2).abs() < 1E-3);
        //         }
        //     }
        //     if p.latitude == 27.5 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 21.6).abs() < 1E-3);
        //         }
        //         if p.longitude == -175.0 {
        //             assert!((p.value - 21.4).abs() < 1E-3);
        //         }
        //     }
        //     if p.latitude == 25.0 {
        //         if p.longitude == -180.0 {
        //             assert!((p.value - 23.8).abs() < 1E-3);
        //         }
        //         if p.longitude == -175.0 {
        //             assert!((p.value - 23.8).abs() < 1E-3);
        //         }
        //         if p.longitude == 170.0 {
        //             assert!((p.value - 23.2).abs() < 1E-3);
        //         }
        //         if p.longitude == 160.0 {
        //             assert!((p.value - 21.8).abs() < 1E-3);
        //         }
        //     }
        // }
    }
}
