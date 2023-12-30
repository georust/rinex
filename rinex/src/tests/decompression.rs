#[cfg(test)]
mod test {
    use crate::hatanaka::Decompressor;
    use crate::tests::toolkit::obsrinex_check_observables;
    use crate::tests::toolkit::random_name;
    use crate::tests::toolkit::test_observation_rinex;
    use crate::{erratic_time_frame, evenly_spaced_time_frame, tests::toolkit::TestTimeFrame};
    use crate::{observable, prelude::*};
    use itertools::Itertools;
    use std::collections::HashMap;
    use std::path::Path;
    use std::str::FromStr;
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
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse CRINEX
            let path = format!("../test_resources/CRNX/V1/{}", crnx_name);
            let crnx = Rinex::from_file(&path);

            assert!(crnx.is_ok());
            let mut rnx = crnx.unwrap();

            let header = rnx.header.obs.as_ref().unwrap();

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
                test_observation_rinex(
                    &rnx,
                    "2.11",
                    Some("MIXED"),
                    "GPS, GLO",
                    "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
                    "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5",
                    Some("2021-01-01T00:00:00 GPST"),
                    Some("2021-01-01T23:59:30 GPST"),
                    evenly_spaced_time_frame!(
                        "2021-01-01T00:00:00 GPST",
                        "2021-01-01T00:09:00 GPST",
                        "30 s"
                    ),
                );
            } else if crnx_name.eq("npaz3550.21d") {
                assert_eq!(infos.version.major, 1);
                assert_eq!(infos.version.minor, 0);
                assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
                assert_eq!(
                    infos.date,
                    Epoch::from_gregorian_utc(2021, 12, 28, 00, 18, 00, 00)
                );

                test_observation_rinex(
                    &rnx,
                    "2.11",
                    Some("MIXED"),
                    "GPS, GLO",
                    "G08,G10,G15,G16,G18,G21,G23,G26,G32,R04,R05,R06,R10,R12,R19,R20,R21",
                    "C1, L1, L2, P2, S1, S2",
                    Some("2021-12-21T00:00:00 GPST"),
                    Some("2021-12-21T23:59:30 GPST"),
                    evenly_spaced_time_frame!(
                        "2021-12-21T00:00:00 GPST",
                        "2021-12-21T01:04:00 GPST",
                        "30 s"
                    ),
                );
            } else if crnx_name.eq("wsra0010.21d") {
                test_observation_rinex(
                    &rnx,
                    "2.11",
                    Some("MIXED"),
                    "GPS, GLO",
                    "R09, R02, G07, G13, R17, R16, R01, G18, G26, G10, G30, G23, G27, G08, R18, G20, R15, G21, G15, R24, G16",
                    "L1, L2, C1, P2, P1, S1, S2",
                    Some("2021-01-01T00:00:00 GPST"),
                    None,
                    evenly_spaced_time_frame!(
                        "2021-01-01T00:00:00 GPST",
                        "2021-01-01T00:08:00 GPST",
                        "30 s"
                    ),
                );
            } else if crnx_name.eq("aopr0010.17d") {
                test_observation_rinex(
                    &rnx,
                    "2.10",
                    Some("GPS"),
                    "GPS",
                    "G31, G27, G03, G32, G16, G08, G14, G23, G22, G26",
                    "C1, L1, L2, P1, P2",
                    Some("2017-01-01T00:00:00 GPST"),
                    None,
                    erratic_time_frame!(
                        "
                        2017-01-01T00:00:00 GPST,
                        2017-01-01T03:33:40 GPST,
                        2017-01-01T06:09:10 GPST
                    "
                    ),
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
                test_observation_rinex(
                    &rnx,
                    "2.11",
                    Some("MIXED"),
                    "GPS, GLO, GAL, EGNOS",
                    "G07, G08, G10, G16, G18, G21, G23, G26, G32, R04, R05, R10, R12, R19, R20, R21, E04, E11, E12, E19, E24, E25, E31, E33, S23, S36",
                    "L1, L2, C1, C2, P1, P2, D1, D2, S1, S2, L5, C5, D5, S5, L7, C7, D7, S7, L8, C8, D8, S8",
                    Some("2021-12-21T00:00:00 GPST"),
                    None,
                    evenly_spaced_time_frame!(
                    "2021-12-21T00:00:00 GPST",
                    "2021-12-21T00:00:30 GPST",
                    "30 s"),
                );
            }
            // decompress and write to file
            rnx.crnx2rnx_mut();
            let filename = format!("{}.rnx", random_name(10));
            assert!(
                rnx.to_file(&filename).is_ok(),
                "failed to dump \"{}\" after decompression",
                crnx_name
            );

            // then run comparison with model
            let obs = rnx.header.obs.as_ref().unwrap();
            assert!(obs.crinex.is_none());

            // parse plain RINEX and run reciprocity
            let path = format!("../test_resources/OBS/V2/{}", rnx_name);
            let model = Rinex::from_file(&path);
            assert!(model.is_ok(), "Failed to parse test model \"{}\"", path);

            //let model = model.unwrap();
            // run testbench
            // test_toolkit::test_against_model(&rnx, &model, &path);

            // remove copy
            let _ = std::fs::remove_file(filename);
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
            //("ESBC00DNK_R_20201770000_01D_30S_MO.crx", "ESBC00DNK_R_20201770000_01D_30S_MO.rnx"),
            //("KMS300DNK_R_20221591000_01H_30S_MO.crx", "KMS300DNK_R_20221591000_01H_30S_MO.rnx"),
            //("MOJN00DNK_R_20201770000_01D_30S_MO.crx", "MOJN00DNK_R_20201770000_01D_30S_MO.rnx"),
        ];
        for duplet in pool {
            let (crnx_name, rnx_name) = duplet;
            // parse CRINEX
            let path = format!("../test_resources/CRNX/V3/{}", crnx_name);
            let crnx = Rinex::from_file(&path);

            assert!(crnx.is_ok());
            let mut rnx = crnx.unwrap();
            assert!(rnx.header.obs.is_some());
            let obs = rnx.header.obs.as_ref().unwrap();
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
            }

            // convert to RINEX
            rnx.crnx2rnx_mut();

            let obs = rnx.header.obs.as_ref().unwrap();
            assert!(obs.crinex.is_none());

            // parse Model for testbench
            let path = format!("../test_resources/OBS/V3/{}", rnx_name);
            let model = Rinex::from_file(&path);
            assert!(model.is_ok(), "Failed to parse test model \"{}\"", path);

            // run testbench
            // test_toolkit::test_against_model(&rnx, &model, &path);
        }
    }
    /*
     * Tries decompression against faulty CRINEX1 content
     */
    #[test]
    fn test_faulty_crinex1() {
        let mut obscodes: HashMap<Constellation, Vec<Observable>> = HashMap::new();
        obscodes.insert(
            Constellation::GPS,
            vec![
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("C1").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("P1").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
            ],
        );
        obscodes.insert(
            Constellation::Glonass,
            vec![
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("C1").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("P1").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
            ],
        );
        let content = "21  1  1  0  0  0.0000000  0 20G07G23G26G20G21G18R24R09G08G27G10G16R18G13R01R16R17G15R02R15";
        let mut decompressor = Decompressor::new();
        assert!(decompressor
            .decompress(1, &Constellation::Mixed, 2, &obscodes, content)
            .is_err());
    }
    #[test]
    fn crnx_v1_zegv0010_21d() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("CRNX")
            .join("V1")
            .join("zegv0010.21d");
        let fullpath = path.to_string_lossy();
        let rnx = Rinex::from_file(fullpath.as_ref());

        assert!(rnx.is_ok(), "failed to parse CRNX/V1/zegv0010.21d");
        let rnx = rnx.unwrap();

        test_observation_rinex(
            &rnx,
            "2.11",
            Some("MIXED"),
            "GPS, GLO",
            "G07, G08, G10, G13, G15, G16, G18, G20, G21, G23, G26, G27, G30, R01, R02, R03, R08, R09, R15, R16, R17, R18, R19, R24",
            "C1, C2, C5, L1, L2, L5, P1, P2, S1, S2, S5",
            Some("2021-01-01T00:00:00 GPST"),
            Some("2021-01-01T23:59:30 GPST"),
            evenly_spaced_time_frame!(
                "2021-01-01T00:00:00 GPST",
                "2021-01-01T00:09:00 GPST",
                "30 s"
            ),
        );

        let record = rnx.record.as_obs().unwrap();

        for (index, ((_e, flag), (clk_offset, vehicles))) in record.iter().enumerate() {
            assert!(flag.is_ok());
            assert!(clk_offset.is_none());
            if index == 0 {
                assert_eq!(vehicles.len(), 24);
                for (sv, observations) in vehicles {
                    if *sv == SV::new(Constellation::GPS, 07) {
                        let mut keys: Vec<_> = observations.keys().cloned().collect();
                        keys.sort();
                        let mut expected: Vec<Observable> = "C1 C2 L1 L2 P1 P2 S1 S2"
                            .split_ascii_whitespace()
                            .map(|k| Observable::from_str(k).unwrap())
                            .collect();
                        expected.sort();
                        let c1 = observations
                            .get(&Observable::from_str("C1").unwrap())
                            .unwrap();
                        assert_eq!(c1.obs, 24178026.635);
                        let c2 = observations
                            .get(&Observable::from_str("C2").unwrap())
                            .unwrap();
                        assert_eq!(c2.obs, 24178024.891);
                        let l1 = observations
                            .get(&Observable::from_str("L1").unwrap())
                            .unwrap();
                        assert_eq!(l1.obs, 127056391.699);
                        let l2 = observations
                            .get(&Observable::from_str("L2").unwrap())
                            .unwrap();
                        assert_eq!(l2.obs, 99004963.017);
                        let p1 = observations
                            .get(&Observable::from_str("P1").unwrap())
                            .unwrap();
                        assert_eq!(p1.obs, 24178026.139);
                        let p2 = observations
                            .get(&Observable::from_str("P2").unwrap())
                            .unwrap();
                        assert_eq!(p2.obs, 24178024.181);
                        let s1 = observations
                            .get(&Observable::from_str("S1").unwrap())
                            .unwrap();
                        assert_eq!(s1.obs, 38.066);
                        let s2 = observations
                            .get(&Observable::from_str("S2").unwrap())
                            .unwrap();
                        assert_eq!(s2.obs, 22.286);
                    } else if *sv == SV::new(Constellation::GPS, 08) {
                        let mut keys: Vec<Observable> = observations.keys().cloned().collect();
                        keys.sort();
                        let mut expected: Vec<Observable> = "C1 C2 C5 L1 L2 L5 P1 P2 S1 S2 S5"
                            .split_ascii_whitespace()
                            .map(|k| Observable::from_str(k).unwrap())
                            .collect();
                        expected.sort();
                        let c1 = observations
                            .get(&Observable::from_str("C1").unwrap())
                            .unwrap();
                        assert_eq!(c1.obs, 21866748.928);
                        let c2 = observations
                            .get(&Observable::from_str("C2").unwrap())
                            .unwrap();
                        assert_eq!(c2.obs, 21866750.407);
                        let c5 = observations
                            .get(&Observable::from_str("C5").unwrap())
                            .unwrap();
                        assert_eq!(c5.obs, 21866747.537);
                        let l1 = observations
                            .get(&Observable::from_str("L1").unwrap())
                            .unwrap();
                        assert_eq!(l1.obs, 114910552.082);
                        let l2 = observations
                            .get(&Observable::from_str("L2").unwrap())
                            .unwrap();
                        assert_eq!(l2.obs, 89540700.326);
                        let l5 = observations
                            .get(&Observable::from_str("L5").unwrap())
                            .unwrap();
                        assert_eq!(l5.obs, 85809828.276);
                        let p1 = observations
                            .get(&Observable::from_str("P1").unwrap())
                            .unwrap();
                        assert_eq!(p1.obs, 21866748.200);
                        let p2 = observations
                            .get(&Observable::from_str("P2").unwrap())
                            .unwrap();
                        assert_eq!(p2.obs, 21866749.482);
                        let s1 = observations
                            .get(&Observable::from_str("S1").unwrap())
                            .unwrap();
                        assert_eq!(s1.obs, 45.759);
                        let s2 = observations
                            .get(&Observable::from_str("S2").unwrap())
                            .unwrap();
                        assert_eq!(s2.obs, 49.525);
                        let s5 = observations
                            .get(&Observable::from_str("S5").unwrap())
                            .unwrap();
                        assert_eq!(s5.obs, 52.161);
                    } else if *sv == SV::new(Constellation::GPS, 13) {
                        let mut keys: Vec<Observable> = observations.keys().cloned().collect();
                        keys.sort();
                        let mut expected: Vec<Observable> = "C1 L1 L2 P1 P2 S1 S2"
                            .split_ascii_whitespace()
                            .map(|k| Observable::from_str(k).unwrap())
                            .collect();
                        expected.sort();
                        assert_eq!(keys, expected);
                        //let c1 = observations.get("C1")
                        //	.unwrap();
                        //assert_eq!(s2.obs, 49.525);
                        //  25107711.730 5                                 131941919.38305 102811868.09001
                        //                  25107711.069 1  25107709.586 1        33.150           8.952
                        let c1 = observations
                            .get(&Observable::from_str("C1").unwrap())
                            .unwrap();
                        assert_eq!(c1.obs, 25107711.730);
                        let l1 = observations
                            .get(&Observable::from_str("L1").unwrap())
                            .unwrap();
                        assert_eq!(l1.obs, 131941919.383);
                        let l2 = observations
                            .get(&Observable::from_str("L2").unwrap())
                            .unwrap();
                        assert_eq!(l2.obs, 102811868.090);
                        let p1 = observations
                            .get(&Observable::from_str("P1").unwrap())
                            .unwrap();
                        assert_eq!(p1.obs, 25107711.069);
                        let p2 = observations
                            .get(&Observable::from_str("P2").unwrap())
                            .unwrap();
                        assert_eq!(p2.obs, 25107709.586);
                        let s1 = observations
                            .get(&Observable::from_str("S1").unwrap())
                            .unwrap();
                        assert_eq!(s1.obs, 33.150);
                        let s2 = observations
                            .get(&Observable::from_str("S2").unwrap())
                            .unwrap();
                        assert_eq!(s2.obs, 8.952);
                    }
                }
            }
        }
    }
    #[test]
    fn v3_acor00esp_r_2021_crx() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("test_resources")
            .join("CRNX")
            .join("V3")
            .join("ACOR00ESP_R_20213550000_01D_30S_MO.crx");
        let fullpath = path.to_string_lossy();
        let crnx = Rinex::from_file(fullpath.as_ref());
        assert!(crnx.is_ok());
        let rnx = crnx.unwrap();

        assert!(rnx.header.obs.is_some());
        let obs = rnx.header.obs.as_ref().unwrap();
        assert!(obs.crinex.is_some());
        let infos = obs.crinex.as_ref().unwrap();

        assert_eq!(infos.version.major, 3);
        assert_eq!(infos.version.minor, 0);
        assert_eq!(infos.prog, "RNX2CRX ver.4.0.7");
        assert_eq!(
            infos.date,
            Epoch::from_gregorian_utc(2021, 12, 28, 01, 01, 00, 00)
        );

        test_observation_rinex(
            &rnx,
            "3.04",
            Some("MIXED"),
            "GPS, GLO, GAL, BDS",
            "G01, G07, G08, G10, G16, G18, G21, G23, G26, G30, R04, R05, R10, R12, R20, R21, E02, E11, E12, E24, E25, E31, E33, E36, C05, C11, C14, C21, C22, C23, C25, C28, C34, C37, C42, C43, C44, C58",
            "C1C, L1C, S1C, C2S, L2S, S2S, C2W, L2W, S2W, C5Q, L5Q, S5Q, C1C, L1C, S1C, C2P, L2P, S2P, C2C, L2C, S2C, C3Q, L3Q, S3Q, C1C, L1C, S1C, C5Q, L5Q, S5Q, C6C, L6C, S6C, C7Q, L7Q, S7Q, C8Q, L8Q, S8Q, C2I, L2I, S2I, C6I, L6I, S6I, C7I, L7I, S7I",
            Some("2021-12-21T00:00:00 GPST"),
            Some("2021-12-21T23:59:30 GPST"),
            evenly_spaced_time_frame!(
                "2021-12-21T00:00:00 GPST",
                "2021-12-21T00:12:00 GPST",
                "30 s"
            ),
        );

        /* G +R +E +C */
        obsrinex_check_observables(
            &rnx,
            Constellation::GPS,
            &[
                "C1C", "L1C", "S1C", "C2S", "L2S", "S2S", "C2W", "L2W", "S2W", "C5Q", "L5Q", "S5Q",
            ],
        );
        obsrinex_check_observables(
            &rnx,
            Constellation::Galileo,
            &[
                "C1C", "L1C", "S1C", "C5Q", "L5Q", "S5Q", "C6C", "L6C", "S6C", "C7Q", "L7Q", "S7Q",
                "C8Q", "L8Q", "S8Q",
            ],
        );
        obsrinex_check_observables(
            &rnx,
            Constellation::Glonass,
            &[
                "C1C", "L1C", "S1C", "C2P", "L2P", "S2P", "C2C", "L2C", "S2C", "C3Q", "L3Q", "S3Q",
            ],
        );
        obsrinex_check_observables(
            &rnx,
            Constellation::BeiDou,
            &[
                "C2I", "L2I", "S2I", "C6I", "L6I", "S6I", "C7I", "L7I", "S7I",
            ],
        );

        /*
         * record test
         */
        let record = rnx.record.as_obs().unwrap();

        for (_, (clk_offset, _)) in record {
            assert!(clk_offset.is_none());
        }

        for (e_index, epoch) in rnx.epoch().enumerate() {
            let (_, vehicles) = record.get(&(epoch, EpochFlag::Ok)).unwrap();
            if e_index == 0 {
                /*
                 * 1st epoch
                 */
                assert_eq!(vehicles.len(), 38);
                let keys: Vec<_> = vehicles.keys().copied().collect();
                let mut expected: Vec<SV> = vec![
                    SV::new(Constellation::GPS, 01),
                    SV::new(Constellation::GPS, 07),
                    SV::new(Constellation::GPS, 08),
                    SV::new(Constellation::GPS, 10),
                    SV::new(Constellation::GPS, 16),
                    SV::new(Constellation::GPS, 18),
                    SV::new(Constellation::GPS, 21),
                    SV::new(Constellation::GPS, 23),
                    SV::new(Constellation::GPS, 26),
                    SV::new(Constellation::GPS, 30),
                    SV::new(Constellation::Glonass, 04),
                    SV::new(Constellation::Glonass, 05),
                    SV::new(Constellation::Glonass, 10),
                    SV::new(Constellation::Glonass, 12),
                    SV::new(Constellation::Glonass, 20),
                    SV::new(Constellation::Glonass, 21),
                    SV::new(Constellation::Galileo, 02),
                    SV::new(Constellation::Galileo, 11),
                    SV::new(Constellation::Galileo, 12),
                    SV::new(Constellation::Galileo, 24),
                    SV::new(Constellation::Galileo, 25),
                    SV::new(Constellation::Galileo, 31),
                    SV::new(Constellation::Galileo, 33),
                    SV::new(Constellation::Galileo, 36),
                    SV::new(Constellation::BeiDou, 05),
                    SV::new(Constellation::BeiDou, 11),
                    SV::new(Constellation::BeiDou, 14),
                    SV::new(Constellation::BeiDou, 21),
                    SV::new(Constellation::BeiDou, 22),
                    SV::new(Constellation::BeiDou, 23),
                    SV::new(Constellation::BeiDou, 25),
                    SV::new(Constellation::BeiDou, 28),
                    SV::new(Constellation::BeiDou, 34),
                    SV::new(Constellation::BeiDou, 37),
                    SV::new(Constellation::BeiDou, 42),
                    SV::new(Constellation::BeiDou, 43),
                    SV::new(Constellation::BeiDou, 44),
                    SV::new(Constellation::BeiDou, 58),
                ];
                expected.sort();
                assert_eq!(keys, expected);
            } else if e_index == rnx.epoch().count() - 1 {
                /*
                 * last epoch
                 */
                assert_eq!(vehicles.len(), 38);
                let keys: Vec<_> = vehicles.keys().copied().collect();
                let mut expected: Vec<SV> = vec![
                    SV::new(Constellation::GPS, 01),
                    SV::new(Constellation::GPS, 07),
                    SV::new(Constellation::GPS, 08),
                    SV::new(Constellation::GPS, 10),
                    SV::new(Constellation::GPS, 16),
                    SV::new(Constellation::GPS, 18),
                    SV::new(Constellation::GPS, 21),
                    SV::new(Constellation::GPS, 23),
                    SV::new(Constellation::GPS, 26),
                    SV::new(Constellation::GPS, 30),
                    SV::new(Constellation::Glonass, 04),
                    SV::new(Constellation::Glonass, 05),
                    SV::new(Constellation::Glonass, 10),
                    SV::new(Constellation::Glonass, 12),
                    SV::new(Constellation::Glonass, 20),
                    SV::new(Constellation::Glonass, 21),
                    SV::new(Constellation::Galileo, 02),
                    SV::new(Constellation::Galileo, 11),
                    SV::new(Constellation::Galileo, 12),
                    SV::new(Constellation::Galileo, 24),
                    SV::new(Constellation::Galileo, 25),
                    SV::new(Constellation::Galileo, 31),
                    SV::new(Constellation::Galileo, 33),
                    SV::new(Constellation::Galileo, 36),
                    SV::new(Constellation::BeiDou, 05),
                    SV::new(Constellation::BeiDou, 11),
                    SV::new(Constellation::BeiDou, 14),
                    SV::new(Constellation::BeiDou, 21),
                    SV::new(Constellation::BeiDou, 22),
                    SV::new(Constellation::BeiDou, 23),
                    SV::new(Constellation::BeiDou, 25),
                    SV::new(Constellation::BeiDou, 28),
                    SV::new(Constellation::BeiDou, 34),
                    SV::new(Constellation::BeiDou, 37),
                    SV::new(Constellation::BeiDou, 42),
                    SV::new(Constellation::BeiDou, 43),
                    SV::new(Constellation::BeiDou, 44),
                    SV::new(Constellation::BeiDou, 58),
                ];
                expected.sort();
                assert_eq!(keys, expected);
                let c58 = vehicles.get(&SV::new(Constellation::BeiDou, 58)).unwrap();

                let mut keys: Vec<Observable> = c58.keys().cloned().collect();
                keys.sort();

                let mut expected: Vec<Observable> = "C2I L2I S2I"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();

                assert_eq!(keys, expected);

                let c2i = c58.get(&Observable::from_str("C2I").unwrap()).unwrap();
                assert_eq!(c2i.obs, 32241892.280);
                assert!(c2i.lli.is_none());
                assert!(c2i.snr.is_none());

                let l2i = c58.get(&Observable::from_str("L2I").unwrap()).unwrap();
                assert_eq!(l2i.obs, 167892006.433);

                let s2i = c58.get(&Observable::from_str("S2I").unwrap()).unwrap();
                assert_eq!(s2i.obs, 47.650);
            }
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_mojn00dnk_sig_strength_regression() {
        let crnx =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz");
        assert!(crnx.is_ok());
        let rnx = crnx.unwrap();
        /*
         * Verify identified observables
         */
        let obs = rnx.header.obs.unwrap().codes.clone();
        for constell in [Constellation::Glonass, Constellation::GPS] {
            let codes = obs.get(&constell);
            assert!(codes.is_some(), "MOJN00DNK_R_20201770000_01D_30S_MO: missing observable codes for constellation {:?}", constell);

            let codes = codes.unwrap();

            let expected: Vec<Observable> = match constell {
                Constellation::Glonass => {
                    vec![
                        observable!("c1c"),
                        observable!("c1p"),
                        observable!("c2c"),
                        observable!("c2p"),
                        observable!("c3q"),
                        observable!("d1c"),
                        observable!("d1p"),
                        observable!("d2c"),
                        observable!("d2p"),
                        observable!("d3q"),
                        observable!("l1c"),
                        observable!("l1p"),
                        observable!("l2c"),
                        observable!("l2p"),
                        observable!("l3q"),
                        observable!("s1c"),
                        observable!("s1p"),
                        observable!("s2c"),
                        observable!("s2p"),
                        observable!("s3q"),
                    ]
                },
                Constellation::GPS => {
                    vec![
                        observable!("c1c"),
                        observable!("c1w"),
                        observable!("c2l"),
                        observable!("c2w"),
                        observable!("c5q"),
                        observable!("d1c"),
                        observable!("d2l"),
                        observable!("d2w"),
                        observable!("d5q"),
                        observable!("l1c"),
                        observable!("l2l"),
                        observable!("l2w"),
                        observable!("l5q"),
                        observable!("s1c"),
                        observable!("s1w"),
                        observable!("s2l"),
                        observable!("s2w"),
                        observable!("s5q"),
                    ]
                },
                _ => todo!("add this constell if you want to test it"),
            };

            if codes.len() != expected.len() {
                panic!("mojn00dnk_r__20201770000_01D_30S_MO: {:?}: idenfied observables \"{:#?}\" - but we expect \"{:#?}\"", constell, codes, expected);
            }
            for i in 0..expected.len() {
                let code = codes.get(i);
                assert!(code.is_some(), "MOJN00DNK_R_20201770000_01D_30S_MO: missing observable \"{:?}\" for constellation {:?}", expected[i], constell);
            }
        }
        /*
         * Record content testing
         */
        let record = rnx.record.as_obs();
        assert!(
            record.is_some(),
            "failed to unwrap MOJN00DNK_R_20201770000_01D_30S_MO as OBS RINEX"
        );

        let record = record.unwrap();
        for (_epoch, (clk_offset, _svs)) in record {
            assert!(clk_offset.is_none());
        }
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_mojn00dnk() {
        let crnx =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz");
        assert!(crnx.is_ok());
        let rnx = crnx.unwrap();

        /* C +E +G +I +J +R +S */
        obsrinex_check_observables(
            &rnx,
            Constellation::BeiDou,
            &[
                "C2I", "C6I", "C7I", "D2I", "D6I", "D7I", "L2I", "L6I", "L7I", "S2I", "S6I", "S7I",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::Galileo,
            &[
                "C1C", "C5Q", "C6C", "C7Q", "C8Q", "D1C", "D5Q", "D6C", "D7Q", "D8Q", "L1C", "L5Q",
                "L6C", "L7Q", "L8Q", "S1C", "S5Q", "S6C", "S7Q", "S8Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::GPS,
            &[
                "C1C", "C1W", "C2L", "C2W", "C5Q", "D1C", "D2L", "D2W", "D5Q", "L1C", "L2L", "L2W",
                "L5Q", "S1C", "S1W", "S2L", "S2W", "S5Q",
            ],
        );

        obsrinex_check_observables(&rnx, Constellation::IRNSS, &["C5A", "D5A", "L5A", "S5A"]);

        obsrinex_check_observables(
            &rnx,
            Constellation::QZSS,
            &[
                "C1C", "C2L", "C5Q", "D1C", "D2L", "D5Q", "L1C", "L2L", "L5Q", "S1C", "S2L", "S5Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::Glonass,
            &[
                "C1C", "C1P", "C2C", "C2P", "C3Q", "D1C", "D1P", "D2C", "D2P", "D3Q", "L1C", "L1P",
                "L2C", "L2P", "L3Q", "S1C", "S1P", "S2C", "S2P", "S3Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::SBAS,
            &["C1C", "C5I", "D1C", "D5I", "L1C", "L5I", "S1C", "S5I"],
        );
    }
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_esbc00dnk() {
        let crnx =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz");
        assert!(crnx.is_ok());
        let rnx = crnx.unwrap();

        /* C +E +G +J +R +S */
        obsrinex_check_observables(
            &rnx,
            Constellation::BeiDou,
            &[
                "C2I", "C6I", "C7I", "D2I", "D6I", "D7I", "L2I", "L6I", "L7I", "S2I", "S6I", "S7I",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::Galileo,
            &[
                "C1C", "C5Q", "C6C", "C7Q", "C8Q", "D1C", "D5Q", "D6C", "D7Q", "D8Q", "L1C", "L5Q",
                "L6C", "L7Q", "L8Q", "S1C", "S5Q", "S6C", "S7Q", "S8Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::GPS,
            &[
                "C1C", "C1W", "C2L", "C2W", "C5Q", "D1C", "D2L", "D2W", "D5Q", "L1C", "L2L", "L2W",
                "L5Q", "S1C", "S1W", "S2L", "S2W", "S5Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::QZSS,
            &[
                "C1C", "C2L", "C5Q", "D1C", "D2L", "D5Q", "L1C", "L2L", "L5Q", "S1C", "S2L", "S5Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::Glonass,
            &[
                "C1C", "C1P", "C2C", "C2P", "C3Q", "D1C", "D1P", "D2C", "D2P", "D3Q", "L1C", "L1P",
                "L2C", "L2P", "L3Q", "S1C", "S1P", "S2C", "S2P", "S3Q",
            ],
        );

        obsrinex_check_observables(
            &rnx,
            Constellation::SBAS,
            &["C1C", "C5I", "D1C", "D5I", "L1C", "L5I", "S1C", "S5I"],
        );
    }
}
