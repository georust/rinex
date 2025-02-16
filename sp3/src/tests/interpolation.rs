//! SP3 interpolation specific tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use hifitime::Unit;
    use std::path::PathBuf;
    use std::str::FromStr;

    #[test]
    #[should_panic]
    #[cfg(feature = "flate2")]
    fn even_interpolation_order() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");

        let sp3 = SP3::from_gzip_file(&path).unwrap();

        let g01 = SV::from_str("G01").unwrap();
        let t0 = Epoch::from_str("2023-08-27T18:00:00 GPST").unwrap();

        let _ = sp3.satellite_position_lagrangian_interpolation(g01, t0, 2);
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn interpolation_feasibility() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("SP3")
            .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");

        let sp3 = SP3::from_gzip_file(&path).unwrap();

        let g01 = SV::from_str("G01").unwrap();
        let g72 = SV::from_str("G72").unwrap();

        let t0 = Epoch::from_str("2023-08-27T18:00:00 GPST").unwrap();
        let t0_min_1s = t0 - 1.0 * Unit::Second;
        let t0_min_5min = t0 - 5.0 * Unit::Minute;
        let t0_5min = t0 + 5.0 * Unit::Second;
        let t0_14min_59s = t0 + 14.0 * Unit::Minute + 59.0 * Unit::Second;

        let t1 = Epoch::from_str("2023-08-27T18:15:00 GPST").unwrap();
        let t1_1s = t1 + 1.0 * Unit::Second;
        let t1_5min = t1 + 5.0 * Unit::Minute;
        let t1_14min_59s = t1 + 14.0 * Unit::Minute + 59.0 * Unit::Second;

        let t2 = Epoch::from_str("2023-08-27T18:30:00 GPST").unwrap();
        let t2_1s = t2 + 1.0 * Unit::Second;
        let t2_7min = t2 + 7.0 * Unit::Minute;
        let t2_14min_59s = t2 + 14.0 * Unit::Minute + 59.0 * Unit::Second;

        let t3 = t2 + 15.0 * Unit::Minute;
        let t3_1s = t3 + 1.0 * Unit::Second;
        let t3_1min = t3 + 1.0 * Unit::Minute;
        let t3_14min_59s = t3 + 14.0 * Unit::Minute + 59.0 * Unit::Second;

        let tn2 = Epoch::from_str("2023-08-29T17:15:00 GPST").unwrap();

        let tn1 = Epoch::from_str("2023-08-29T17:30:00 GPST").unwrap();
        let tn1_min_1s = tn1 - 1.0 * Unit::Second;

        let tn3 = tn2 - 15.0 * Unit::Minute;
        let tn3_min_1s = tn3 - 1.0 * Unit::Second;
        let tn3_min_7min = tn3 - 7.0 * Unit::Minute;

        let tn4 = tn3 - 15.0 * Unit::Minute;

        let tn = Epoch::from_str("2023-08-29T17:45:00 GPST").unwrap();
        let tn_min_1s = tn - 1.0 * Unit::Second;

        // test: invalid SV
        assert!(sp3
            .satellite_position_lagrangian_interpolation(g72, t0, 7)
            .is_none());

        assert!(sp3
            .satellite_position_lagrangian_interpolation(g72, t1, 7)
            .is_none());

        assert!(sp3
            .satellite_position_lagrangian_interpolation(g72, t2, 7)
            .is_none());

        // test: too early (x3)
        for t in [t0_min_1s, t0_min_5min, t0, t0_5min, t0_14min_59s] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 3)
                .is_none());
        }

        // test: first x3 feasible
        for t in [t1_1s, t1_5min, t1_14min_59s, t2] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 3)
                .is_some());
        }

        // test: second x3 feasible
        for t in [t2_1s, t2_7min, t2_14min_59s] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 3)
                .is_some());
        }

        // test: last feasible (x3)
        for t in [tn1, tn1_min_1s] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 3)
                .is_some());
        }

        // test: too late (x3)
        for t in [tn, tn_min_1s] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 3)
                .is_none());
        }

        // test: too early x7
        for t in [
            t0_min_1s,
            t0_min_5min,
            t0,
            t0_5min,
            t0_14min_59s,
            t1,
            t1_1s,
            t1_5min,
            t1_14min_59s,
            t2,
            t2_14min_59s,
            t3,
        ] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 7)
                .is_none());
        }

        // test: first feasible x7
        for t in [t3_1s, t3_1min, t3_14min_59s] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 7)
                .is_some());
        }

        // test: too late (x7)
        for t in [tn, tn_min_1s, tn1, tn2] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 7)
                .is_none());
        }

        // test: last feasible (x7)
        for t in [tn3, tn3_min_1s, tn3_min_7min, tn4] {
            assert!(sp3
                .satellite_position_lagrangian_interpolation(g01, t, 7)
                .is_some());
        }
    }
}
