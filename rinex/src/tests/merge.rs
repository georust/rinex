#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::tests::toolkit::test_observation_rinex;
    use crate::Merge;
    use crate::{
        //erratic_time_frame,
        evenly_spaced_time_frame,
        tests::toolkit::TestTimeFrame,
    };
    //use itertools::Itertools;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    fn fail_on_type_mismatch() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");
        let path1 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");
        let path2 = test_resources
            .clone()
            .join("OBS")
            .join("V3")
            .join("LARM0630.22O");
        let mut r1 = Rinex::from_file(&path1.to_string_lossy()).unwrap();
        let r2 = Rinex::from_file(&path2.to_string_lossy()).unwrap();
        assert!(r1.merge_mut(&r2).is_err())
    }
    #[test]
    fn merge_nav() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");
        let path1 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("AMEL00NLD_R_20210010000_01D_MN.rnx");
        let rnx_a = Rinex::from_file(&path1.to_string_lossy());
        assert!(
            rnx_a.is_ok(),
            "failed to parse NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
        );
        let path2 = test_resources
            .clone()
            .join("NAV")
            .join("V3")
            .join("CBW100NLD_R_20210010000_01D_MN.rnx");
        let rnx_b = Rinex::from_file(&path2.to_string_lossy());
        assert!(
            rnx_b.is_ok(),
            "failed to parse NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx"
        );

        let rnx_a = rnx_a.unwrap();
        let rnx_b = rnx_b.unwrap();
        let merged = rnx_a.merge(&rnx_b);
        assert!(
            merged.is_ok(),
            "failed to merge NAV/V3/CBW100NLD_R_20210010000_01D_MN.rnx into NAV/V3/AMEL00NLD_R_20210010000_01D_MN.rnx"
        );

        // dump
        let merged = merged.unwrap();
        assert!(
            merged.to_file("merge.txt").is_ok(),
            "failed to generate file previously merged"
        );
        assert!(
            merged.is_merged(),
            "is_merged() should be true after merging!"
        );

        // parse back
        let rnx = Rinex::from_file("merge.txt");
        assert!(rnx.is_ok(), "Failed to parsed back previously merged file");

        let rnx = rnx.unwrap();
        assert!(
            rnx.is_merged(),
            "failed to identify a merged file correctly"
        );

        /*
         * Unlock reciprocity test in near future
         *  NAV file production does not work correctly at the moment,
         *  due to formatting issues
         */
        // assert_eq!(rnx, merged, "Merge::ops reciprocity");

        // remove file we just generated
        let _ = std::fs::remove_file("merge.txt");
    }
    #[test]
    #[ignore]
    fn merge_obs() {
        let test_resources = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources");
        let path1 = test_resources
            .clone()
            .join("OBS")
            .join("V2")
            .join("AJAC3550.21O");
        let rnx_a = Rinex::from_file(&path1.to_string_lossy());
        assert!(rnx_a.is_ok(), "failed to parse OBS/V2/AJAC3550.21O");
        let path2 = test_resources
            .clone()
            .join("OBS")
            .join("V2")
            .join("npaz3550.21o");
        let rnx_b = Rinex::from_file(&path2.to_string_lossy());
        assert!(rnx_b.is_ok(), "failed to parse OBS/V2/npaz3550.21o");

        let rnx_a = rnx_a.unwrap();
        let rnx_b = rnx_b.unwrap();
        let merged = rnx_a.merge(&rnx_b);
        assert!(
            merged.is_ok(),
            "failed to merge OBS/V2/npaz3550.21o into OBS/V2/AJAC3550.21O"
        );

        let merged = merged.unwrap();

        test_observation_rinex(
            &merged,
            "2.11",
            Some("MIXED"),
            "GPS, GLO, GAL, EGNOS",
            "G07, G08, G10, G15, G16, G18, G21, G23, G26, G32, R04, R05, R06, R10, R12, R19, R20, R21, E04, E11, E12, E19, E24, E25, E31, E33, S23, S36",
            "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8",
            Some("2021-21-12T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            evenly_spaced_time_frame!(
            "2021-12-21T00:00:00 GPST",
            "2021-12-21T01:04:00 GPST",
            "30 s")
        );

        // dump
        assert!(
            merged.to_file("merge.txt").is_ok(),
            "failed to generate file previously merged"
        );
        assert!(
            merged.is_merged(),
            "is_merged() should be true after merging!"
        );

        // parse back
        let rnx = Rinex::from_file("merge.txt");
        assert!(rnx.is_ok(), "Failed to parsed back previously merged file");

        let rnx = rnx.unwrap();
        assert!(
            rnx.is_merged(),
            "failed to identify a merged file correctly"
        );

        assert_eq!(rnx, merged, "merge() reciprocity");

        // remove file we just generated
        let _ = std::fs::remove_file("merge.txt");
    }
    #[cfg(feature = "antex")]
    use crate::antex::antenna::AntennaMatcher;
    #[cfg(feature = "antex")]
    use crate::Carrier;
    #[test]
    #[cfg(feature = "flate2")]
    #[cfg(feature = "antex")]
    fn merge_atx() {
        let fp = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/ATX/V1/TROSAR25.R4__LEIT_2020_09_23.atx";
        let rinex_a = Rinex::from_file(&fp);
        let rinex_a = rinex_a.unwrap();

        let fp =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/ATX/V1/igs14_small.atx.gz";
        let rinex_b = Rinex::from_file(&fp);
        let rinex_b = rinex_b.unwrap();

        let merged = rinex_a.merge(&rinex_b);
        assert!(merged.is_ok(), "merged atx(a,b) failed");
        let merged = merged.unwrap();

        let antennas: Vec<_> = merged.antennas().collect();
        assert_eq!(antennas.len(), 7, "bad number of antennas");

        for (name, expected_apc) in [
            ("JPSLEGANT_E", (1.36, -0.43, 35.44)),
            ("JPSODYSSEY_I", (1.06, -2.43, 70.34)),
            ("TROSAR25.R4", (-0.22, -0.01, 154.88)),
        ] {
            let fakenow = Epoch::from_gregorian_utc_at_midnight(2023, 01, 01);
            let apc = merged.rx_antenna_apc_offset(
                fakenow,
                AntennaMatcher::IGSCode(name.to_string()),
                Carrier::L1,
            );
            assert!(apc.is_some(), "APC should still be contained after merge()");
            assert_eq!(apc.unwrap(), expected_apc);
        }
    }
}
