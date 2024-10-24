#[cfg(test)]
mod test {
    use crate::{
        prelude::{GeodeticMarker, Rinex},
        tests::toolkit::{generic_observation_rinex_test, TimeFrame},
    };

    use std::path::Path;

    #[test]
    fn v2_aopr0010_17o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("aopr0010.17o");

        let fullpath = path.to_string_lossy();

        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "2.10",
            Some("GPS"),
            false,
            "G31,G27,G03,G32,G16,G14,G08,G23,G22,G07, G30, G11, G19, G07",
            "GPS",
            &[("GPS", "L1, L2, C1, P1, P2")],
            Some("2017-01-01T00:00:00 GPST"),
            None,
            None,
            None,
            None,
            TimeFrame::from_erratic_csv(
                "2017-01-01T00:00:00 GPST,    
                2017-01-01T03:33:40 GPST,
                2017-01-01T06:09:10 GPST",
            ),
            vec![],
            vec![],
        );
    }

    #[test]
    fn v2_npaz3550_21o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("npaz3550.21o");
        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "2.11",
            Some("MIXED"),
            false,
            "G08,G10,G15,G16,G18,G21,G23,G26,G32,R04,R05,R06,R10,R12,R19,R20,R21",
            "GPS, GLO",
            &[
                ("GPS", "C1, L1, L2, P2, S1, S2"),
                ("GLO", "C1, L1, L2, P2, S1, S2"),
            ],
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv(
                "2021-12-21T00:00:00 GPST, 2021-12-21T01:04:00 GPST, 30 s",
            ),
            vec![],
            vec![],
        );
    }

    #[test]
    fn v2_rovn0010_21o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("rovn0010.21o");

        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "2.11",
            Some("MIXED"), 
            false,
            "G07, G08, G10, G13, G15, G16, G18, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5", 
            &[
                ("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
                ("GLO", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
            ],
            Some("2021-01-01T00:00:00 GPST"), 
            Some("2021-01-01T23:59:30 GPST"), 
            Some((3859571.8076,  413007.6749, 5044091.5729)),
            Some("Hans van der Marel"),
            Some(GeodeticMarker::default().with_number("13544M001")),

            TimeFrame::from_erratic_csv("
                2021-01-01T00:00:00 GPST,
                2021-01-01T00:00:30 GPST,
                2021-01-01T01:10:00 GPST,
                2021-01-01T02:25:00 GPST,
                2021-01-01T02:25:30 GPST,
                2021-01-01T02:26:00 GPST
            ",
            ),
            vec![],
            vec![],
        );

        assert_eq!(dut.header.agency, "TU Delft for Deltares");
    }
    #[test]
    fn v3_duth0630() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V3")
            .join("DUTH0630.22O");

        let fullpath = path.to_string_lossy();

        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "3.02",
            Some("MIXED"),
            false,
            "G03, G01, G04, G09, G17, G19, G21, G22, G31, G32, R01, R02, R08, R09, R10, R17, R23, R24",
            "GPS, GLO",
            &[
                ("GPS", "C1C, L1C, D1C, S1C, C2W, L2W, D2W, S2W"),
                ("GLO", "C1C, L1C, D1C, S1C, C2P, L2P, D2P, S2P"),
            ],
            Some("2022-03-04T00:00:00 GPST"),
            Some("2022-03-04T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_erratic_csv("2022-03-04T00:00:00 GPST, 2022-03-04T00:28:30 GPST, 2022-03-04T00:57:00 GPST"),
            vec![],
            vec![],
        );
    }

    #[test]
    fn v4_kms300dnk_r_2022_v3crx() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx";

        let dut = Rinex::from_file(&test_resource).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "2.10",
            Some("MIXED"),
            false,
            "G01",
            "GPS",
            &[("GPS", "C1")],
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T00:00:00 GPST"),
            Some((1.0, 2.0, 3.0)),
            Some("hello"),
            Some(GeodeticMarker::default().with_name("Test")),
            TimeFrame::from_inclusive_csv(
                "2021-01-01T00:00:00 GPST, 2021-01-01T00:00:00 GPST, 30 s",
            ),
            vec![],
            vec![],
        );
    }

    #[test]
    #[ignore]
    fn v2_kosg0010_95o() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("KOSG0010.95O");

        let fullpath = path.to_string_lossy();
        let rnx = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &rnx,
            None,
            "2.0",
            Some("GPS"),
            false,
            "G01, G04, G05, G06, G16, G17, G18, G19, G20, G21, G22, G23, G24, G25, G27, G29, G31",
            "GPS",
            &[("GPS", "C1, L1, L2, P2, S1")],
            Some("1995-01-01T00:00:00 GPST"),
            Some("1995-01-01T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_erratic_csv(
                "
            1995-01-01T00:00:00 GPST,
            1995-01-01T11:00:00 GPST,
            1995-01-01T20:44:30 GPST
        ",
            ),
            vec![],
            vec![],
        );
    }

    #[test]
    fn v2_ajac3550() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V2")
            .join("AJAC3550.21O");
        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "2.10",
            Some("MIXED"),
            false,
            "G01",
            "GPS, GAL, GLO, SBAS",
            &[("GPS", "L1"), ("GLO", "L1"), ("SBAS", "L1"), ("Gal", "L1")],
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T00:00:00 GPST"),
            None,
            None,
            None,
            TimeFrame::from_erratic_csv("2021-01-01T00:00:00 GPST"),
            vec![],
            vec![],
        );
    }

    #[test]
    fn v3_noa10630() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("OBS")
            .join("V3")
            .join("NOA10630.22O");
        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref()).unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "3.02",
            Some("GPS"),
            false,
            "G01, G03, G09, G17, G19, G21, G22",
            "GPS",
            &[("GPS", "C1C, L1C, D1C, S1C, S2W, L2W, D2W, S2W")],
            Some("2022-03-04T00:00:00 GPST"),
            Some("2022-03-04T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_erratic_csv(
                "2022-03-04T00:00:00 GPST,
            2022-03-04T00:00:30 GPST,
            2022-03-04T00:01:00 GPST,
            2022-03-04T00:52:30 GPST",
            ),
            vec![],
            vec![],
        );
    }
}
