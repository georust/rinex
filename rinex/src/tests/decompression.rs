#[cfg(test)]
mod test {
    use crate::{
        hatanaka::{Decompressor, Error},
        prelude::{Epoch, EpochFlag, GeodeticMarker, MarkerType, Observable, Rinex, SV},
        tests::toolkit::{generic_observation_rinex_test, random_name, SignalDataPoint, TimeFrame},
    };
    use std::{
        fs::{remove_file as fs_remove_file, File},
        io::Read,
        path::Path,
        str::FromStr,
    };

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
            let crnx = Rinex::from_file::<5>(&path);

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

            // TODO
            let filename = format!("{}.rnx", random_name(10));

            // assert!(
            //     dut.to_file(&filename).is_ok(),
            //     "failed to dump \"{}\" after decompression",
            //     crnx_name
            // );

            // run test on generated file
            let path = format!("../test_resources/OBS/V2/{}", rnx_name);
            let _model = Rinex::from_file::<5>(&path).unwrap();

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

            let mut dut = Rinex::from_file::<5>(&path).unwrap();

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
            let model = Rinex::from_file::<5>(&path).unwrap();

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
        let dut = Rinex::from_file::<5>(fullpath.as_ref()).unwrap();

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
        let dut = Rinex::from_file::<5>(fullpath.as_ref()).unwrap();

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

    #[cfg(feature = "flate2")]
    #[test]
    fn v3_esbc00dnk() {
        let dut = Rinex::from_file::<5>(
            "../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
        )
        .unwrap();

        let mut geo_marker = GeodeticMarker::default()
            .with_name("ESBC00DNK")
            .with_number("10118M001");

        geo_marker.marker_type = Some(MarkerType::Geodetic);

        generic_observation_rinex_test(
            &dut,
            None,
            "3.05",
            Some("MIXED"),
            false,
            "C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C16, C19, C20, C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, C32, C33, C34, C35, C36, C37,
                 E01, E02, E03, E04, E05, E07, E08, E09, E11, E12, E13, E15, E19, E21, E24, E25, E26, E27, E30, E31, E33, E36,
                 G01, G02, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13, G14, G15, G16, G17, G18, G19, G20, G21, G22, G24, G25, G26, G27, G28, G29, G30, G31, G32,
                 R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R23, R24,
                 J01, J02, J03,
                 S23, S25, S26, S36, S44",
            "BDS, GAL, GLO, QZSS, GPS, EGNOS, SDCM, BDSBAS",
            &[
                ("GPS", "C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q, S1C, S1W, S2L, S2W, S5Q"),
                ("BDS", "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I"),
                ("GAL", "C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C, L7Q, L8Q, S1C, S5Q, S6C, S7Q, S8Q"),
                ("GLO", "C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C, L2P, L3Q, S1C, S1P, S2C, S2P, S3Q"),
                ("SBAS", "C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I"),
                ("QZSS", "C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q"),
            ],
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            None,
            None,
            Some(geo_marker),
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
        let dut = Rinex::from_file::<5>(
            "../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz",
        )
        .unwrap();

        generic_observation_rinex_test(
            &dut,
            None,
            "3.05",
            Some("MIXED"),
            false,
            "
                G01, G02, G03, G04, G05, G06, G07, G08, G09, G10, G11, G12, G13, G14, G15, G16, G17, G18, G19, G20, G21, G22, G24, 
                G25, G26, G27, G28, G29, G30, G31, G32,
                C05, C06, C07, C08, C09, C10, C11, C12, C13, C14, C16, C19, C20, C21, C22, C23, C24, C25, C26, C27, C28, C29, C30, 
                C32, C33, C34, C35, C36, C37,
                R01, R02, R03, R04, R05, R06, R07, R08, R09, R10, R11, R12, R13, R14, R15, R16, R17, R18, R19, R20, R21, R23, R24,
                E01, E02, E03, E04, E05, E07, E08, E09, E11, E12, E13, E15, E19, E21, E24, E25, E26, E27, E30, E31, E33, E36,
                J01, J02, J03,
                I01, I02, I04, I05, I06, I09,
                S23, S25, S26, S27, S36, S44
            ",
            "GPS, GLO, BDS, GAL, QZSS, IRNSS, EGNOS, GAGAN, BDSBAS, SDCM",
            &[
                ("GPS", "C1C, C1W, C2L, C2W, C5Q, D1C, D2L, D2W, D5Q, L1C, L2L, L2W, L5Q, S1C, S1W, S2L, S2W, S5Q"),
                ("BDS", "C2I, C6I, C7I, D2I, D6I, D7I, L2I, L6I, L7I, S2I, S6I, S7I"),
                ("Gal", "C1C, C5Q, C6C, C7Q, C8Q, D1C, D5Q, D6C, D7Q, D8Q, L1C, L5Q, L6C, L7Q, L8Q, S1C, S5Q, S6C, S7Q, S8Q"),
                ("IRNSS", "C5A, D5A, L5A, S5A"),
                ("QZSS", "C1C, C2L, C5Q, D1C, D2L, D5Q, L1C, L2L, L5Q, S1C, S2L, S5Q"),
                ("GLO", "C1C, C1P, C2C, C2P, C3Q, D1C, D1P, D2C, D2P, D3Q, L1C, L1P, L2C, L2P, L3Q, S1C, S1P, S2C, S2P, S3Q"),
                ("SBAS", "C1C, C5I, D1C, D5I, L1C, L5I, S1C, S5I"),
            ],
            Some("2020-06-25T00:00:00 GPST"),
            Some("2020-06-25T23:59:30 GPST"),
            None,
            None,
            None,
            TimeFrame::from_inclusive_csv("2020-06-25T00:00:00 GPST, 2020-06-25T23:59:30 GPST, 30 s"),
            vec![],
            vec![]
        );
    }

    #[test]
    fn v1_aopr0017d_raw() {
        let mut last_passed = false;
        let mut nth = 1;
        let mut buf = [0; 4096];
        let fd = File::open("../test_resources/CRNX/V1/aopr0010.17d").unwrap();
        let mut decompressor = Decompressor::<File>::new(fd);

        // consume entire file
        loop {
            match decompressor.read(&mut buf) {
                Err(_) => {},
                Ok(size) => {
                    if size == 0 {
                        break; // EOS
                    }
                    assert!(size <= 4096);

                    let mut ptr = 0;
                    let buf_str = String::from_utf8_lossy(&buf[..size]);

                    for line in buf_str.lines() {
                        match nth {
                            1 => {
                                assert_eq!(line, "     2.10           OBSERVATION DATA    G (GPS)             RINEX VERSION / TYPE");
                            },
                            2 => {
                                assert_eq!(line, "teqc  2002Mar14     Arecibo Observatory 20170102 06:00:02UTCPGM / RUN BY / DATE");
                            },
                            3 => {
                                assert_eq!(
                                    line,
                                    "Linux 2.0.36|Pentium II|gcc|Linux|486/DX+                   COMMENT"
                                );
                            },
                            13 => {
                                assert_eq!(line, "     5    L1    L2    C1    P1    P2                        # / TYPES OF OBSERV");
                            },
                            14 => {
                                assert_eq!(
                                    line,
                                    "Version: Version:                                           COMMENT"
                                );
                            },
                            18 => {
                                assert_eq!(line, "  2017     1     1     0     0    0.0000000     GPS         TIME OF FIRST OBS");
                            },
                            19 => {
                                assert_eq!(
                                    line,
                                    "                                                            END OF HEADER"
                                );
                            },
                            20 => {
                                assert_eq!(
                                    line,
                                    " 17  1  1  0  0  0.0000000  0 10G31G27G 3G32G16G 8G14G23G22G26"
                                );
                            },
                            21 => {
                                assert_eq!(line, " -14746974.73049 -11440396.20948  22513484.6374   22513484.7724   22513487.3704 ");
                            },
                            22 => {
                                assert_eq!(line, " -19651355.72649 -15259372.67949  21319698.6624   21319698.7504   21319703.7964 ");
                            },
                            23 => {
                                assert_eq!(line, "  -9440000.26548  -7293824.59347  23189944.5874   23189944.9994   23189951.4644 ");
                            },
                            24 => {
                                assert_eq!(line, " -11141744.16748  -8631423.58147  23553953.9014   23553953.6364   23553960.7164 ");
                            },
                            25 => {
                                assert_eq!(line, " -21846711.60849 -16970657.69649  20528865.5524   20528865.0214   20528868.5944 ");
                            },
                            26 => {
                                assert_eq!(line, "  -2919082.75648  -2211037.84947  24165234.9594   24165234.7844   24165241.6424 ");
                            },
                            27 => {
                                assert_eq!(line, " -20247177.70149 -15753542.44648  21289883.9064   21289883.7434   21289887.2614 ");
                            },
                            28 => {
                                assert_eq!(line, " -15110614.77049 -11762797.21948  23262395.0794   23262394.3684   23262395.3424 ");
                            },
                            29 => {
                                assert_eq!(line, " -16331314.56648 -12447068.51348  22920988.2144   22920987.5494   22920990.0634 ");
                            },
                            30 => {
                                assert_eq!(line, " -15834397.66049 -12290568.98049  21540206.1654   21540206.1564   21540211.9414 ");
                            },
                            31 => {
                                assert_eq!(
                                    line,
                                    " 17  1  1  3 33 40.0000000  0  9G30G27G11G16G 8G 7G23G 9G 1   "
                                );
                            },
                            32 => {
                                assert_eq!(line, "  -4980733.18548  -3805623.87347  24352349.1684   24352347.9244   24352356.1564 ");
                            },
                            33 => {
                                assert_eq!(line, "  -9710828.79748  -7513506.68548  23211317.1574   23211317.5034   23211324.2834 ");
                            },
                            34 => {
                                assert_eq!(line, " -26591640.60049 -20663619.71349  20668830.8234   20668830.4204   20668833.2334 ");
                            },
                            38 => {
                                assert_eq!(line, " -18143490.68049 -14126079.68448  22685259.0754   22685258.3664   22685261.2134 ");
                            },
                            39 => {
                                assert_eq!(line, " -16594887.53049 -12883140.10148  22336785.6934   22336785.4334   22336790.8924 ");
                            },
                            40 => {
                                assert_eq!(line, " -19095445.86249 -14826971.50648  21708306.6584   21708306.5704   21708312.9414 ");
                            },
                            41 => {
                                assert_eq!(
                                    line,
                                    " 17  1  1  6  9 10.0000000  0 11G30G17G 3G11G19G 8G 7G 6G22G28G 1"
                                );
                            },
                            42 => {
                                assert_eq!(line, " -23668184.66249 -18367274.15149  20796245.2334   20796244.8234   20796250.6334 ");
                            },
                            43 => {
                                assert_eq!(line, "  -5877878.73348  -4575160.53248  23410058.5724   23410059.2714   23410062.1064 ");
                            },
                            52 => {
                                last_passed = true;
                                assert_eq!(line, " -21848286.72849 -16972039.81549  21184456.3894   21184456.9144   21184462.1224 ");
                            },
                            _ => {},
                        }
                        nth += 1;
                        ptr += line.len() + 1;
                    }

                    if ptr < buf_str.len() {
                        let remainder = buf_str[ptr..].to_string();
                        println!("remainder \"{}\"", remainder);
                        // buf.clear();
                    }
                },
            }
        }
        assert!(last_passed, "nth={}", nth);
    }

    #[test]
    fn v1_zegv0010_raw() {
        let mut last_passed = false;
        let mut nth = 1;
        let mut buf = [0; 4096];
        let fd = File::open("../test_resources/CRNX/V1/zegv0010.21d").unwrap();
        let mut decompressor = Decompressor::<File>::new(fd);

        loop {
            match decompressor.read(&mut buf) {
                Ok(size) => {
                    if size == 0 {
                        break; // EOS
                    }
                    assert!(size <= 4096);

                    let mut ptr = 0;
                    let buf_str = String::from_utf8_lossy(&buf[..size]);

                    for line in buf_str.lines() {
                        match nth {
                            1 => {
                                assert_eq!(line, "     2.11           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                            },
                            2 => {
                                assert_eq!(line, "ssrcrin-13.4.4x                         20210101 000000 UTC PGM / RUN BY / DATE");
                            },
                            10 => {
                                assert_eq!(line, "-------------------------------------                       COMMENT");
                            },
                            11 => {
                                assert_eq!(line, "    11    C1    C2    C5    L1    L2    L5    P1    P2    S1# / TYPES OF OBSERV");
                            },
                            12 => {
                                assert_eq!(line, "          S2    S5                                          # / TYPES OF OBSERV");
                            },
                            13 => {
                                assert_eq!(line, "    54                                                      # OF SATELLITES");
                            },
                            14 => {
                                assert_eq!(line, "   G01  1020  1033  1036   990   984  1036   984   984  1020PRN / # OF OBS");
                            },
                            32 => {
                                assert_eq!(line, "   G10  1147  1145  1145  1129  1126  1145  1126  1126  1147PRN / # OF OBS");
                            },
                            40 => {
                                assert_eq!(line, "   G15  1066  1057        1061  1059        1060  1060  1066PRN / # OF OBS");
                            },
                            45 => {
                                assert_eq!(line, "        1157                                                PRN / # OF OBS");
                            },
                            50 => {
                                assert_eq!(line, "   G20  1154              1120  1113        1113  1113  1154PRN / # OF OBS");
                            },
                            52 => {
                                assert_eq!(line, "   G21  1044              1003   989         989   989  1044PRN / # OF OBS");
                            },
                            53 => {
                                assert_eq!(line, "         989                                                PRN / # OF OBS");
                            },
                            56 => {
                                assert_eq!(line, "   G23  1161  1160  1160  1149  1146  1160  1146  1146  1161PRN / # OF OBS");
                            },
                            104 => {
                                assert_eq!(line, "   R16  1254  1255        1239  1229                    1254PRN / # OF OBS");
                            },
                            105 => {
                                assert_eq!(line, "        1255                                                PRN / # OF OBS");
                            },
                            106 => {
                                assert_eq!(line, "   R17  1285  1285        1283  1283                    1285PRN / # OF OBS");
                            },
                            107 => {
                                assert_eq!(line, "        1285                                                PRN / # OF OBS");
                            },
                            122 => {
                                assert_eq!(line, "    30.000                                                  INTERVAL");
                            },
                            123 => {
                                assert_eq!(line, "  2021     1     1     0     0    0.0000000     GPS         TIME OF FIRST OBS");
                            },
                            124 => {
                                assert_eq!(line, "  2021     1     1    23    59   30.0000000     GPS         TIME OF LAST OBS");
                            },
                            125 => {
                                assert_eq!(line, "                                                            END OF HEADER");
                            },
                            126 => {
                                assert_eq!(line, " 21 01 01 00 00 00.0000000  0 24G07G08G10G13G15G16G18G20G21G23G26G27");
                            },
                            127 => {
                                assert_eq!(line, "                                G30R01R02R03R08R09R15R16R17R18R19R24");
                            },
                            128 => {
                                assert_eq!(line, "  24178026.635 6  24178024.891 6                 127056391.69906  99004963.01703");
                            },
                            129 => {
                                assert_eq!(line, "                  24178026.139 3  24178024.181 3        38.066          22.286");
                            },
                            130 => {
                                panic!("DONE");
                            },
                            1493 => {
                                assert_eq!(line, "  23573585.517 7  23573589.832 5                 126058552.95807  98045562.02905");
                            },
                            1494 => {
                                assert_eq!(line, "                                                        42.626          35.171  ");
                            },
                            1495 => last_passed = true,
                            _ => {},
                        }
                        nth += 1;
                        ptr += line.len() + 1;
                    }

                    if ptr < buf_str.len() {
                        let remainder = buf_str[ptr..].to_string();
                        println!("remainder \"{}\"", remainder);
                        // buf.clear();
                    }
                },
                Err(e) => {
                    println!("i/o error: {}", e);
                },
            }
        }
        assert_eq!(nth, 1495);
        assert!(last_passed, "nth={}", nth);
    }

    #[test]
    fn v3_duth0630_raw() {
        let mut last_passed = false;
        let mut nth = 1;
        let mut buf = [0; 4096];
        let fd = File::open("../test_resources/CRNX/V3/DUTH0630.22D").unwrap();
        let mut decompressor = Decompressor::<File>::new(fd);

        // consume entire file
        loop {
            match decompressor.read(&mut buf) {
                Err(e) => {},
                Ok(size) => {
                    if size == 0 {
                        break; // EOS
                    }
                    assert!(size <= 4096);

                    let mut ptr = 0;
                    let buf_str = String::from_utf8_lossy(&buf[..size]);

                    for line in buf_str.lines() {
                        match nth {
                            1 => {
                                assert_eq!(line, "     3.02           OBSERVATION DATA    M: MIXED            RINEX VERSION / TYPE");
                            },
                            2 => {
                                assert_eq!(line, "HEADER CHANGED BY EPN CB ON 2022-03-11                      COMMENT");
                            },
                            3 => {
                                assert_eq!(
                                    line,
                                    "TO BE CONFORM WITH THE INFORMATION IN                       COMMENT"
                                );
                            },
                            35 => {
                                assert_eq!(line, "                                                            END OF HEADER");
                            },
                            36 => {
                                assert_eq!(line, "> 2022 03 04 00 00  0.0000000  0 18");
                            },
                            37 => {
                                assert_eq!(line, "G01  20243517.560   106380411.41808     -1242.766          51.250    20243518.680    82893846.80009      -968.395          54.750 ");
                            },
                            89 => {
                                assert_eq!(line, "R23  22543866.020   120594470.51907     -4464.453          44.25");
                            },
                            90 => {
                                assert_eq!(line, "R24  20147683.700   107738728.87108     -2188.113          51.000    20147688.700    83796794.50808     -1701.871          48.500");
                                last_passed = true;
                            },
                            _ => {},
                        }
                        nth += 1;
                        ptr += line.len() + 1;
                    }

                    if ptr < buf_str.len() {
                        let remainder = buf_str[ptr..].to_string();
                        println!("remainder \"{}\"", remainder);
                        // buf.clear();
                    }
                },
            }
        }
        assert_eq!(nth, 90);
        assert!(last_passed, "nth={}", nth);
    }

    #[test]
    fn v3_esbcdnk_raw() {
        let mut last_passed = false;
        let mut nth = 1;
        let mut buf = [0; 4096];
        let fd = File::open("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
            .unwrap();
        let mut decompressor = Decompressor::<File>::new_gzip(fd);

        // consume entire file
        loop {
            match decompressor.read(&mut buf) {
                Err(e) => {},
                Ok(size) => {
                    if size == 0 {
                        break; // EOS
                    }
                    assert!(size <= 4096);

                    let mut ptr = 0;
                    let buf_str = String::from_utf8_lossy(&buf[..size]);

                    for line in buf_str.lines() {
                        match nth {
                            1 => {
                                assert_eq!(line, "     3.05           OBSERVATION DATA    M (MIXED)           RINEX VERSION / TYPE");
                            },
                            2 => {
                                assert_eq!(line, "sbf2rin-13.4.5                          20220706 130812 UTC PGM / RUN BY / DATE");
                            },
                            3 => {
                                assert_eq!(line, "gfzrnx-1.16-8177    FILE MERGE          20220706 132211 UTC COMMENT");
                            },
                            4 => {
                                assert_eq!(
                                    line,
                                    "ESBC00DNK                                                   MARKER NAME"
                                );
                            },
                            133734 => {
                                assert_eq!(line, "44  41519088.701 5                        99.299 5                 218184295.26505                        35.500");
                                last_passed = true;
                            },
                            _ => {},
                        }
                        nth += 1;
                        ptr += line.len() + 1;
                    }

                    if ptr < buf_str.len() {
                        let remainder = buf_str[ptr..].to_string();
                        println!("remainder \"{}\"", remainder);
                        // buf.clear();
                    }
                },
            }
        }
        assert!(last_passed, "nth={}", nth);
    }
}
