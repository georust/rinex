// Decimation specific tests
#[cfg(test)]
mod decimation {
    use crate::prelude::*;
    use crate::preprocessing::*;
    //use itertools::Itertools;
    use std::path::Path;

    #[test]
    #[cfg(feature = "flate2")]
    fn obs_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("CRNX")
            .join("V3")
            .join("ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let mut rinex = rinex.unwrap();
        let len = rinex.epoch().count();

        rinex.decimate_by_interval_mut(Duration::from_seconds(60.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len / 2, "decimate(1'): error",);

        rinex.decimate_by_interval_mut(Duration::from_seconds(60.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len / 2, "decimate(1'): error",);

        rinex.decimate_by_interval_mut(Duration::from_seconds(120.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len / 4, "decimate(2'): error",);
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("MET")
            .join("V3")
            .join("POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let mut rinex = rinex.unwrap();
        let len = rinex.epoch().count();

        rinex.decimate_by_interval_mut(Duration::from_seconds(60.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len, "decimate(1'): error",);

        rinex.decimate_by_interval_mut(Duration::from_seconds(360.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len / 2, "decimate(6'): error",);

        rinex.decimate_by_interval_mut(Duration::from_seconds(900.0));
        let count = rinex.epoch().count();
        assert_eq!(count, len / 4, "decimate(15'): error",);
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn nav_decimation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("NAV")
            .join("V3")
            .join("ESBC00DNK_R_20201770000_01D_MN.rnx.gz");

        let fullpath = path.to_string_lossy();
        let rinex = Rinex::from_file(fullpath.as_ref());
        assert!(rinex.is_ok(), "failed to parse \"{}\"", fullpath);

        let mut rinex = rinex.unwrap();
        let _len = rinex.epoch().count();

        rinex.decimate_by_interval_mut(Duration::from_seconds(60.0));
        let count = rinex.epoch().count();
        assert_eq!(count, 1016, "decimate(1'): error",);

        rinex.decimate_by_interval_mut(Duration::from_seconds(61.0));
        let count = rinex.epoch().count();
        assert_eq!(count, 1016, "decimate(1'+1s): error",);
    }
}
