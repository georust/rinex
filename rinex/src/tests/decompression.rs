#[cfg(test)]
mod test {
    use crate::{
        prelude::{Epoch, EpochFlag, GeodeticMarker, Observable, Rinex, SV},
        tests::toolkit::{
            generic_observation_rinex_against_model, generic_observation_rinex_test, random_name,
            ClockDataPoint, SignalDataPoint, TimeFrame,
        },
    };
    use itertools::Itertools;
    use std::{collections::HashMap, fs::remove_file as fs_remove_file, path::Path, str::FromStr};
    #[test]
    fn testbench_v1() {
        let pool = vec![
            ("zegv0010.21d", "zegv0010.21o"),
            ("AJAC3550.21D", "AJAC3550.21O"),
            //("KOSG0010.95D", "KOSG0010.95O"), //TODO@ fix tests/obs/v2_kosg first
            ("aopr0010.17d", "aopr0010.17o"),
            ("npaz3550.21d", "npaz3550.21o"),
            ("wsra0010.21d", "wsra0010.21o"),
        ];
        for (crnx_name, rnx_name) in pool {
            // parse DUT
            let path = format!("../test_resources/CRNX/V1/{}", crnx_name);
            let crnx = Rinex::from_file(&path);

            assert!(crnx.is_ok(), "failed to parse {}", path);
            let mut dut = crnx.unwrap();

            let header = dut.header.obs.as_ref().unwrap();

            assert!(header.crinex.is_some());
            let infos = header.crinex.as_ref().unwrap();

            if crnx_name.eq("zegv0010.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(
                    infos.date,
                    Epoch::from_gregorian_utc(2021, 01, 02, 00, 01, 00, 00)
                );
                generic_observation_rinex_test(
                    &dut,
                    None,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "GPS, GLO",
                    "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
                    &[
                        ("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
                        ("GLO", "C1"),
                    ],
                    Some("2021-01-01T00:00:00 GPST"),
                    Some("2021-01-01T23:59:30 GPST"),
                    None,
                    None,
                    None,
                    TimeFrame::from_exclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:09:00 GPST, 30 s"),
                    vec![],
                    vec![],
                );
            } else if crnx_name.eq("npaz3550.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(
                    infos.date,
                    Epoch::from_gregorian_utc(2021, 12, 28, 00, 18, 00, 00)
                );
                generic_observation_rinex_test(
                    &dut,
                    None,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "GPS, GLO",
                    "G08,G10,G15,G16,G18,G21,G23,G26,G32,R04,R05,R06,R10,R12,R19,R20,R21",
                    &[("GPS", "C1, L1, L2, P2, S1, S2")],
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
            } else if crnx_name.eq("wsra0010.21d") {
                generic_observation_rinex_test(
                    &dut,
                    None,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "GPS, GLO",
                    "R09, R02, G07, G13, R17, R16, R01, G18, G26, G10, G30, G23, G27, G08, R18, G20, R15, G21, G15, R24, G16",
                    &[
                        ("GPS", "L1, L2, C1, P2, P1, S1, S2"),
                        ("GPS", "L1, L2, C1, P2, P1, S1, S2"),
                    ],
                    Some("2021-01-01T00:00:00 GPST"),
                    None,
                    None,
                    None,
                    None,
                    TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:08:00 GPST, 30s"),
                    vec![],
                    vec![],
                );
            } else if crnx_name.eq("aopr0010.17d") {
                generic_observation_rinex_test(
                    &dut,
                    None,
                    "2.10",
                    Some("GPS"),
                    false,
                    "GPS",
                    "G31, G27, G03, G32, G16, G08, G14, G23, G22, G26",
                    &[("GPS", "C1, L1, L2, P1, P2"), ("GLO", "C1")],
                    Some("2017-01-01T00:00:00 GPST"),
                    None,
                    None,
                    None,
                    None,
                    TimeFrame::from_erratic_csv(
                        "
                    2017-01-01T00:00:00 GPST,
                    2017-01-01T03:33:40 GPST,
                    2017-01-01T06:09:10 GPST
                    ",
                    ),
                    vec![],
                    vec![],
                );
            //} else if crnx_name.eq("KOSG0010.95D") {
            //    test_observation_rinex(
            //        &rnx,
            //        "2.0",
            //        Some("GPS"),
            //        "GPS",
            //        "G01, G04, G05, G06, G16, G17, G18, G19, G20, G21, G22, G23, G24, G25, G27, G29, G31",
            //        "C1, L1, L2, P2, S1",
            //        Some("1995-01-01T00:00:00 GPST"),
            //        Some("1995-01-01T23:59:30 GPST"),
            //        erratic_time_frame!("
            //            1995-01-01T00:00:00 GPST,
            //            1995-01-01T11:00:00 GPST,
            //            1995-01-01T20:44:30 GPST
            //        "),
            //    );
            } else if crnx_name.eq("AJAC3550.21D") {
                generic_observation_rinex_test(
                    &dut,
                    None,
                    "2.11",
                    Some("MIXED"),
                    false,
                    "GPS, GLO, GAL, EGNOS",
                    "G07, G08, G10, G16, G18, G21, G23, G26, G32, R04, R05, R10, R12, R19, R20, R21, E04, E11, E12, E19, E24, E25, E31, E33, S23, S36",
                    &[
                        ("GPS", "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8"),
                        ("GLO", "C1"),
                        ("GAL", "C1"),
                        ("EGNOS", "C1"),
                    ],
                    Some("2021-12-21T00:00:00 GPST"),
                    None,
                    None,
                    None,
                    None,
                    TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T00:00:30 GPST, 30 s"),
                    vec![],
                    vec![],
                );
            }

            // decompress and write to file
            dut.crnx2rnx_mut();

            let specs = dut.header.obs.as_ref().unwrap();
            assert!(specs.crinex.is_none());

            let filename = format!("{}.rnx", random_name(10));
            assert!(
                dut.to_file(&filename).is_ok(),
                "failed to dump \"{}\" after decompression",
                crnx_name
            );

            // run test on generated file
            let path = format!("../test_resources/OBS/V2/{}", rnx_name);
            let model = Rinex::from_file(&path).unwrap();

            // TODO unlock this
            // generic_observation_rinex_against_model();

            let _ = fs_remove_file(filename); // cleanup
        }
    }
    #[test]
    fn testbench_v3() {
        let pool = vec![
            ("DUTH0630.22D", "DUTH0630.22O"),
            (
                "ACOR00ESP_R_20213550000_01D_30S_MO.crx",
                "ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            ),
            ("pdel0010.21d", "pdel0010.21o"),
            ("flrs0010.12d", "flrs0010.12o"),
            ("VLNS0010.22D", "VLNS0010.22O"),
            ("VLNS0630.22D", "VLNS0630.22O"),
            //TODO unlock this
            //("ESBC00DNK_R_20201770000_01D_30S_MO.crx", "ESBC00DNK_R_20201770000_01D_30S_MO.rnx"),
            //("KMS300DNK_R_20221591000_01H_30S_MO.crx", "KMS300DNK_R_20221591000_01H_30S_MO.rnx"),
            //("MOJN00DNK_R_20201770000_01D_30S_MO.crx", "MOJN00DNK_R_20201770000_01D_30S_MO.rnx"),
        ];
        for (crnx_name, rnx_name) in pool {
            // parse DUT
            let path = format!("../test_resources/CRNX/V3/{}", crnx_name);
            let dut = Rinex::from_file(&path);
            assert!(dut.is_ok(), "failed to parse {}", path);
            let mut dut = dut.unwrap();

            assert!(dut.header.obs.is_some());
            let obs = dut.header.obs.as_ref().unwrap();

            assert!(obs.crinex.is_some());
            let infos = obs.crinex.as_ref().unwrap();

            if crnx_name.eq("ACOR00ESP_R_20213550000_01D_30S_MO.crx") {
                assert_eq!(infos.version.major, 3);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(
                    infos.date,
                    Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00)
                );

                // TODO: run detailed testbench
            }

            // convert to RINEX
            dut.crnx2rnx_mut();

            let obs = dut.header.obs.as_ref().unwrap();
            assert!(obs.crinex.is_none());

            // run test on generated file
            let path = format!("../test_resources/OBS/V3/{}", rnx_name);
            let model = Rinex::from_file(&path).unwrap();

            // TODO unlock this
            // generic_observation_rinex_against_model();
        }
    }

    #[test]
    fn v1_zegv0010_21d() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("CRNX")
            .join("V1")
            .join("zegv0010.21d");

        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref());

        assert!(dut.is_ok(), "failed to parse CRNX/V1/zegv0010.21d");
        let rnx = dut.unwrap();

        generic_observation_rinex_test(
            &rnx,
            None,
            "2.11",
            Some("MIXED"),
            false,
            "GPS, GLO",
            "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
            &[
                ("GPS", "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5"),
            ],
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:09:00 GPST, 30 s"),
            vec![
                SignalDataPoint::new(
                    Epoch::from_str("2021-01-01T00:00:00 GPST").unwrap(),
                    EpochFlag::Ok,
                    SV::from_str("G07").unwrap(),
                    Observable::from_str("C1").unwrap(),
                    0.0,
                ),
            ],
            vec![],
        );
    }

    #[test]
    fn v3_acor00esp_r2021() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("CRNX")
            .join("V3")
            .join("ACOR00ESP_R_20213550000_01D_30S_MO.crx");

        let fullpath = path.to_string_lossy();
        let dut = Rinex::from_file(fullpath.as_ref());
        let dut = dut.unwrap();

        assert!(dut.header.obs.is_some());
        let obs = dut.header.obs.as_ref().unwrap();
        assert!(obs.crinex.is_some());

        let infos = obs.crinex.as_ref().unwrap();

        assert_eq!(infos.version.major, 3);
        assert_eq!(infos.version.minor, 0);
        assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
        assert_eq!(
            infos.date,
            Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00)
        );

        generic_observation_rinex_test(
            &dut,
            None,
            "3.04",
            Some("MIXED"),
            false,
            "G01, G07, G08, G10, G16, G18, G21, G23, G26, G30, R04, R05, R10, R12, R20, R21, E02, E11, E12, E24, E25, E31, E33, E36, C05, C11, C14, C21, C22, C23, C25, C28, C34, C37, C42, C43, C44, C58",
            "GPS, GLO, GAL, BDS",
            &[
                ("GPS", "C1C, L1C, S1C, C2S, L2S, S2S, C2W, L2W, S2W, C5Q, L5Q, S5Q"),
                ("Gal", "C1C, L1C, S1C, C5Q, L5Q, S5Q, C6C, L6C, S6C, C7Q, L7Q, S7Q, C8Q, L8Q, S8Q"),
                ("GLO", "C1C, L1C, S1C, C2P, L2P, S2P, C2C, L2C, S2C, C3Q, L3Q, S3Q"),
                ("BDS", "C2I, L2I, S2I, C6I, L6I, S6I, C7I, L7I, S7I"),
            ],
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-12-21T00:00:00 GPST, 2021-12-21T00:12:00 GPST, 30 s"),
            vec![

            ],
            vec![],
        );
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn v3_esbc00dnk() {
        let dut =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "3.05",
            Some("MIXED"),
            false,
            "C05, C07, C10, C12, C19, C20, C23, C32, C34, C37,
                 E01, E03, E05, E09, E13, E15, E24, E31,
                 G02, G05, G07, G08, G09, G13, G15, G18, G21, G27, G28, G30,
                 R01, R02, R08, R09, R10, R11, R12, R17, R18, R19,
                 S23, S25, S36",
            "BDS, GAL, GLO, QZSS, GPS, EGNOS, SDCM, BDSBAS",
            &[
                ("GPS", "L1"),
                ("BDS", "C1C"),
                ("GAL", "C2I"),
                ("GLO", "C1C"),
                ("SBAS", "C1C"),
            ],
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            None,
            None,
            Some(
                GeodeticMarker::default()
                    .with_name("ESBC00DNK")
                    .with_number("10118M001"),
            ),
            TimeFrame::from_inclusive_csv(
                "2020-06-25T00:00:00 GPST, 2020-06-25T23:59:30 GPST, 30 s",
            ),
            vec![],
            vec![],
        );
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn v3_mojn00dnk() {
        let dut =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "3.10",
            Some("MIXED"),
            false,
            "G01, G02",
            "GPS, GLO",
            &[
                ("GPS", "C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q, S1C, S1W, S2L, S2W, S5Q"),
                ("BDS", "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I"),
                ("Gal", "C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C, L7Q, L8Q, S1C, S5Q, S6C, S7Q, S8Q"),
                ("IRNSS", "C5A, D5A, L5A, S5A"),
                ("QZSS", "C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q"),
                ("GLO", "C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C, L2P, L3Q, S1C, S1P, S2C, S2P, S3Q"),
                ("SBAS", "C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I"),
            ],
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T00:00:00 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2021-01-01T00:00:00 GPST, 2021-01-01T00:00:00GPST, 30 s"),
            vec![],
            vec![]
        );
    }
}
