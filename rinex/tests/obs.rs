#[cfg(test)]
mod test {
    use rinex::observable;
    use rinex::sv;
    use rinex::{header::*, observation::*, prelude::*};
    use std::str::FromStr;
    /*
     * Helper: to create a list of observable
     */
    fn create_observ_list(descriptors: Vec<&str>) -> Vec<Observable> {
        let mut r: Vec<Observable> = vec![];
        for desc in descriptors {
            if desc.starts_with("L") {
                let obs = Observable::Phase(String::from(desc));
                r.push(obs.clone());
            } else if desc.starts_with("P") {
                let obs = Observable::PseudoRange(String::from(desc));
                r.push(obs.clone());
            } else if desc.starts_with("C") {
                let obs = Observable::PseudoRange(String::from(desc));
                r.push(obs.clone());
            } else if desc.starts_with("S") {
                let obs = Observable::SSI(String::from(desc));
                r.push(obs.clone());
            }
        }
        r.sort(); // for comparison purposes
        r
    }
    /*
     * General testbench
     * shared accross all Observation files
     */
    fn testbench(
        rnx: &Rinex,
        _major: u8,
        _minor: u8,
        c: Constellation,
        epochs: Vec<Epoch>,
        observables: Vec<Observable>,
    ) {
        // must have dedicated fields
        assert!(rnx.header.obs.is_some());
        /*
         * Test epoch parsing and identification
         */
        assert!(rnx.epoch().eq(epochs), "parsed wrong epoch content");

        let mut parsed_observables: Vec<Observable> = rnx.observable().cloned().collect();
        parsed_observables.sort();

        assert!(
            observables == parsed_observables,
            "parsed wrong observable content,expecting\n{:?}\ngot\n{:?}",
            observables,
            parsed_observables
        );

        /*
         * Test Record content
         */
        let record = rnx.record.as_obs();
        assert!(record.is_some());
        let record = record.unwrap();
        assert!(record.len() > 0);
        for ((_, _), (clk_offset, vehicles)) in record {
            /*
             * We don't have any files with clock offsets as of today
             */
            assert!(clk_offset.is_none());
            /*
             * test GNSS identification
             */
            if c != Constellation::Mixed {
                for (sv, _) in vehicles {
                    assert_eq!(sv.constellation, c);
                }
            }
        }
    }
    #[test]
    fn v2_aopr0010_17o() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/aopr0010.17o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();

        let epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2017, 01, 01, 0, 0, 0, 0),
            Epoch::from_gregorian_utc(2017, 01, 01, 3, 33, 40, 0),
            Epoch::from_gregorian_utc(2017, 01, 01, 6, 9, 10, 0),
        ];

        let observables = create_observ_list(vec!["L1", "L2", "P1", "P2", "C1"]);

        testbench(&rinex, 2, 11, Constellation::GPS, epochs, observables);
        let record = rinex.record.as_obs().unwrap();

        for (index, (_e, (_, vehicles))) in record.iter().enumerate() {
            let keys: Vec<_> = vehicles.keys().collect();
            if index == 0 {
                assert_eq!(
                    keys,
                    vec![
                        &Sv::new(Constellation::GPS, 03),
                        &Sv::new(Constellation::GPS, 08),
                        &Sv::new(Constellation::GPS, 14),
                        &Sv::new(Constellation::GPS, 16),
                        &Sv::new(Constellation::GPS, 22),
                        &Sv::new(Constellation::GPS, 23),
                        &Sv::new(Constellation::GPS, 26),
                        &Sv::new(Constellation::GPS, 27),
                        &Sv::new(Constellation::GPS, 31),
                        &Sv::new(Constellation::GPS, 32),
                    ]
                );

                /*
                 * Test G03
                 */
                let sv = Sv::new(Constellation::GPS, 03);
                let observations = vehicles.get(&sv).unwrap();
                let l1 = observations
                    .get(&Observable::from_str("L1").unwrap())
                    .unwrap();
                assert_eq!(l1.obs, -9440000.265);
                assert!(l1.lli.unwrap().intersects(LliFlags::UNDER_ANTI_SPOOFING));
                assert_eq!(l1.snr, Some(Snr::DbHz48_53));

                let l2 = observations
                    .get(&Observable::from_str("L2").unwrap())
                    .unwrap();
                assert_eq!(l2.obs, -7293824.593);
                assert!(l2.lli.unwrap().intersects(LliFlags::UNDER_ANTI_SPOOFING));
                assert_eq!(l2.snr, Some(Snr::DbHz42_47));

                let c1 = observations
                    .get(&Observable::from_str("C1").unwrap())
                    .unwrap();
                assert_eq!(c1.obs, 23189944.587);
                assert!(c1.lli.unwrap().intersects(LliFlags::UNDER_ANTI_SPOOFING));
                assert!(c1.snr.is_none());

                let p1 = observations
                    .get(&Observable::from_str("P1").unwrap())
                    .unwrap();
                assert_eq!(p1.obs, 23189944.999);
                assert!(p1.lli.unwrap().intersects(LliFlags::UNDER_ANTI_SPOOFING));
                assert!(p1.snr.is_none());

                let p2 = observations
                    .get(&Observable::from_str("P2").unwrap())
                    .unwrap();
                assert_eq!(p2.obs, 23189951.464);
                assert!(p2.lli.unwrap().intersects(LliFlags::UNDER_ANTI_SPOOFING));
                assert!(p2.snr.is_none());
            } else if index == 1 {
                assert_eq!(
                    keys,
                    vec![
                        &Sv::new(Constellation::GPS, 01),
                        &Sv::new(Constellation::GPS, 07),
                        &Sv::new(Constellation::GPS, 08),
                        &Sv::new(Constellation::GPS, 09),
                        &Sv::new(Constellation::GPS, 11),
                        &Sv::new(Constellation::GPS, 16),
                        &Sv::new(Constellation::GPS, 23),
                        &Sv::new(Constellation::GPS, 27),
                        &Sv::new(Constellation::GPS, 30),
                    ]
                );
            } else if index == 2 {
                assert_eq!(
                    keys,
                    vec![
                        &Sv::new(Constellation::GPS, 01),
                        &Sv::new(Constellation::GPS, 03),
                        &Sv::new(Constellation::GPS, 06),
                        &Sv::new(Constellation::GPS, 07),
                        &Sv::new(Constellation::GPS, 08),
                        &Sv::new(Constellation::GPS, 11),
                        &Sv::new(Constellation::GPS, 17),
                        &Sv::new(Constellation::GPS, 19),
                        &Sv::new(Constellation::GPS, 22),
                        &Sv::new(Constellation::GPS, 28),
                        &Sv::new(Constellation::GPS, 30),
                    ]
                );
            }
        }
    }
    #[test]
    fn v2_npaz3550_21o() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/npaz3550.21o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        //testbench(&rinex, 2, 11, Constellation::Mixed, epochs);

        let obs_hd = rinex.header.obs.as_ref().unwrap();
        let record = rinex.record.as_obs();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();

        //////////////////////////////
        // This file is GPS + GLONASS
        //////////////////////////////
        let obscodes = obs_hd.codes.get(&Constellation::GPS);
        assert_eq!(obscodes.is_some(), true);
        let obscodes = obscodes.unwrap();
        assert_eq!(
            obscodes,
            &vec![
                Observable::from_str("C1").unwrap(),
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
            ]
        );
        let obscodes = obs_hd.codes.get(&Constellation::Glonass);
        assert_eq!(obscodes.is_some(), true);
        let obscodes = obscodes.unwrap();
        assert_eq!(
            obscodes,
            &vec![
                Observable::from_str("C1").unwrap(),
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
            ]
        );

        // test epoch [1]
        let epoch = Epoch::from_gregorian_utc(2021, 12, 21, 0, 0, 0, 0);
        let flag = EpochFlag::Ok;
        let epoch = record.get(&(epoch, flag));
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 17);

        // G08
        let sv = Sv {
            constellation: Constellation::GPS,
            prn: 08,
        };
        let observations = epoch.get(&sv);
        assert_eq!(observations.is_some(), true);
        let observations = observations.unwrap();

        // C1
        let observed = observations.get(&Observable::from_str("C1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 22288985.512);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        // L1
        let observed = observations.get(&Observable::from_str("L1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 117129399.048);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::DbHz36_41));
        // L2
        let observed = observations.get(&Observable::from_str("L2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 91269672.416);
        assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
        assert_eq!(observed.snr, Some(Snr::DbHz36_41));
        // P2
        let observed = observations.get(&Observable::from_str("P2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 22288987.972);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        // S1
        let observed = observations.get(&Observable::from_str("S1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 44.000);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        // S2
        let observed = observations.get(&Observable::from_str("S2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 27.000);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);

        //R19
        let sv = Sv {
            constellation: Constellation::Glonass,
            prn: 19,
        };
        let observations = epoch.get(&sv);
        assert_eq!(observations.is_some(), true);
        let observations = observations.unwrap();

        // C1
        let observed = observations.get(&Observable::from_str("C1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 23250776.648);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        // L1
        let observed = observations.get(&Observable::from_str("L1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 124375967.254);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::DbHz0));
        // L2
        let observed = observations.get(&Observable::from_str("L2").unwrap());
        assert_eq!(observed.is_none(), true);
        // P2
        let observed = observations.get(&Observable::from_str("P2").unwrap());
        assert_eq!(observed.is_none(), true);
        // S1
        let observed = observations.get(&Observable::from_str("S1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 32.000);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        // S2
        let observed = observations.get(&Observable::from_str("S2").unwrap());
        assert_eq!(observed.is_none(), true);
    }
    #[test]
    fn v2_rovn0010_21o() {
        let test_resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V2/rovn0010.21o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();

        /*
         * Header tb
         */
        let header = &rinex.header;
        assert!(rinex.is_observation_rinex());
        assert!(header.obs.is_some());
        assert!(header.meteo.is_none());
        assert_eq!(
            header.ground_position,
            Some(GroundPosition::from_ecef_wgs84((
                3859571.8076,
                413007.6749,
                5044091.5729
            )))
        );
        assert_eq!(header.station_id, "13544M001");
        assert_eq!(header.observer, "Hans van der Marel");
        assert_eq!(header.agency, "TU Delft for Deltares");

        let obs_hd = header.obs.as_ref();
        assert!(obs_hd.is_some());
        let obs_hd = obs_hd.unwrap();

        let record = rinex.record.as_obs();
        assert!(record.is_some());
        let record = record.unwrap();
        //////////////////////////////
        // This file is GPS + GLONASS
        //////////////////////////////
        let obscodes = obs_hd.codes.get(&Constellation::GPS);
        assert_eq!(obscodes.is_some(), true);
        let obscodes = obscodes.unwrap();
        assert_eq!(
            obscodes,
            &vec![
                Observable::from_str("C1").unwrap(),
                Observable::from_str("C2").unwrap(),
                Observable::from_str("C5").unwrap(),
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("L5").unwrap(),
                Observable::from_str("P1").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
                Observable::from_str("S5").unwrap(),
            ]
        );

        let obscodes = obs_hd.codes.get(&Constellation::Glonass);
        assert_eq!(obscodes.is_some(), true);
        let obscodes = obscodes.unwrap();
        assert_eq!(
            obscodes,
            &vec![
                Observable::from_str("C1").unwrap(),
                Observable::from_str("C2").unwrap(),
                Observable::from_str("C5").unwrap(),
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("L5").unwrap(),
                Observable::from_str("P1").unwrap(),
                Observable::from_str("P2").unwrap(),
                Observable::from_str("S1").unwrap(),
                Observable::from_str("S2").unwrap(),
                Observable::from_str("S5").unwrap(),
            ]
        );

        // test epoch [1]
        let epoch = Epoch::from_gregorian_utc(2021, 01, 01, 0, 0, 0, 0);
        let epoch = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 24);

        // G07
        let sv = Sv {
            constellation: Constellation::GPS,
            prn: 07,
        };
        let observations = epoch.get(&sv);
        assert_eq!(observations.is_some(), true);
        let observations = observations.unwrap();

        // C1
        let observed = observations.get(&Observable::from_str("C1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 24225566.040);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::DbHz36_41));
        //C2
        let observed = observations.get(&Observable::from_str("C2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 24225562.932);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        //C5 [missing]
        let observed = observations.get(&Observable::from_str("C5").unwrap());
        assert_eq!(observed.is_none(), true);
        //L1
        let observed = observations.get(&Observable::from_str("L1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 127306204.852);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        //L2
        let observed = observations.get(&Observable::from_str("L2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 99199629.819);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::from_str("4").unwrap()));
        //L5 [missing]
        let observed = observations.get(&Observable::from_str("L5").unwrap());
        assert_eq!(observed.is_none(), true);
        //P1
        let observed = observations.get(&Observable::from_str("P1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 24225565.620);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::from_str("4").unwrap()));
        //P2
        let observed = observations.get(&Observable::from_str("P2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 24225563.191);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::from_str("4").unwrap()));
        //S1
        let observed = observations.get(&Observable::from_str("S1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 40.586);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        //S2
        let observed = observations.get(&Observable::from_str("S2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 25.564);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        //S5 (missing)
        let observed = observations.get(&Observable::from_str("S5").unwrap());
        assert_eq!(observed.is_none(), true);

        // G07
        let sv = Sv {
            constellation: Constellation::Glonass,
            prn: 24,
        };
        let observations = epoch.get(&sv);
        assert_eq!(observations.is_some(), true);
        let observations = observations.unwrap();

        //C1,C2,C5
        let observed = observations.get(&Observable::from_str("C1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 23126824.976);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        let observed = observations.get(&Observable::from_str("C2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 23126830.088);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        let observed = observations.get(&Observable::from_str("C5").unwrap());
        assert_eq!(observed.is_none(), true);
        //L1,L2,L5
        let observed = observations.get(&Observable::from_str("L1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 123669526.377);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        let observed = observations.get(&Observable::from_str("L2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        //assert_eq!(observed.obs, 96187435.849);
        assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(observed.snr, Some(Snr::from_str("6").unwrap()));
        let observed = observations.get(&Observable::from_str("L5").unwrap());
        assert_eq!(observed.is_none(), true);
        //P1, P2
        let observed = observations.get(&Observable::from_str("P1").unwrap());
        assert_eq!(observed.is_none(), true);
        let observed = observations.get(&Observable::from_str("P2").unwrap());
        assert_eq!(observed.is_none(), true);
        //S1,S2,S5
        let observed = observations.get(&Observable::from_str("S1").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 41.931);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        let observed = observations.get(&Observable::from_str("S2").unwrap());
        assert_eq!(observed.is_some(), true);
        let observed = observed.unwrap();
        assert_eq!(observed.obs, 39.856);
        assert_eq!(observed.lli, None);
        assert_eq!(observed.snr, None);
        let observed = observations.get(&Observable::from_str("S5").unwrap());
        assert_eq!(observed.is_none(), true);
    }
    #[test]
    fn v3_duth0630() {
        let resource =
            env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/OBS/V3/DUTH0630.22O";
        let rinex = Rinex::from_file(&resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.header.obs.is_some(), true);
        let obs = rinex.header.obs.as_ref().unwrap();

        /*
         * test Glonass observables
         */
        let observables = obs.codes.get(&Constellation::Glonass);
        assert_eq!(observables.is_some(), true);
        let mut observables = observables.unwrap().clone();
        observables.sort();

        let mut expected: Vec<Observable> = "C1C L1C D1C S1C C2P L2P D2P S2P"
            .split_ascii_whitespace()
            .map(|k| Observable::from_str(k).unwrap())
            .collect();
        expected.sort();
        assert_eq!(observables, expected);

        /*
         * test GPS observables
         */
        let observables = obs.codes.get(&Constellation::GPS);
        assert_eq!(observables.is_some(), true);
        let mut observables = observables.unwrap().clone();
        observables.sort();

        let mut expected: Vec<Observable> = "C1C L1C D1C S1C C2W L2W D2W S2W"
            .split_ascii_whitespace()
            .map(|k| Observable::from_str(k).unwrap())
            .collect();
        expected.sort();
        assert_eq!(observables, expected);

        let record = rinex.record.as_obs();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();

        /*
         * Test epochs
         */
        let expected: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 28, 30, 00),
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 57, 00, 00),
        ];
        assert!(
            rinex.epoch().collect::<Vec<Epoch>>() == expected,
            "parsed wrong epoch content"
        );

        let epoch = Epoch::from_gregorian_utc(2022, 03, 04, 0, 0, 0, 0);
        let e = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(e.is_some(), true);
        let (clk, vehicles) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicles.len(), 18);

        let g01 = Sv {
            constellation: Constellation::GPS,
            prn: 01,
        };
        let g01 = vehicles.get(&g01);
        assert_eq!(g01.is_some(), true);
        let data = g01.unwrap();
        let c1c = data.get(&Observable::from_str("C1C").unwrap());
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 20243517.560);
        assert!(c1c.lli.is_none());
        assert!(c1c.snr.is_none());

        let l1c = data.get(&Observable::from_str("L1C").unwrap());
        assert_eq!(l1c.is_some(), true);
        let l1c = l1c.unwrap();
        assert_eq!(l1c.obs, 106380411.418);
        assert_eq!(l1c.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(l1c.snr, Some(Snr::from_str("8").unwrap()));

        let s1c = data.get(&Observable::from_str("S1C").unwrap());
        assert_eq!(s1c.is_some(), true);
        let s1c = s1c.unwrap();
        assert_eq!(s1c.obs, 51.250);
        assert!(s1c.lli.is_none());
        assert!(s1c.snr.is_none());

        let g03 = Sv {
            constellation: Constellation::GPS,
            prn: 03,
        };
        let g03 = vehicles.get(&g03);
        assert_eq!(g03.is_some(), true);
        let data = g03.unwrap();
        let c1c = data.get(&Observable::from_str("C1C").unwrap());
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 20619020.680);
        assert_eq!(c1c.lli.is_none(), true);
        assert_eq!(c1c.snr.is_none(), true);

        let l1c = data.get(&Observable::from_str("L1C").unwrap());
        assert_eq!(l1c.is_some(), true);

        let g04 = Sv {
            constellation: Constellation::GPS,
            prn: 04,
        };
        let g04 = vehicles.get(&g04);
        assert_eq!(g04.is_some(), true);
        let data = g04.unwrap();
        let c1c = data.get(&Observable::from_str("C1C").unwrap());
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 21542633.500);
        assert_eq!(c1c.lli.is_none(), true);
        assert_eq!(c1c.snr.is_none(), true);

        let l1c = data.get(&Observable::from_str("L1C").unwrap());
        assert_eq!(l1c.is_some(), true);

        let epoch = Epoch::from_gregorian_utc(2022, 03, 04, 00, 28, 30, 00);
        let e = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(e.is_some(), true);
        let (clk, vehicles) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicles.len(), 17);

        let epoch = Epoch::from_gregorian_utc(2022, 03, 04, 00, 57, 0, 0);
        let e = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(e.is_some(), true);
        let (clk, vehicles) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicles.len(), 17);
    }
    //#[test]
    fn v4_kms300dnk_r_2022_v3crx() {
        let test_resource = env!("CARGO_MANIFEST_DIR").to_owned()
            + "/../test_resources/CRNX/V3/KMS300DNK_R_20221591000_01H_30S_MO.crx";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        //////////////////////////
        // Header testbench
        //////////////////////////
        assert_eq!(rinex.is_observation_rinex(), true);
        assert_eq!(rinex.header.obs.is_some(), true);
        let obs = rinex.header.obs.as_ref().unwrap();
        let glo_observables = obs.codes.get(&Constellation::Glonass);
        assert_eq!(glo_observables.is_some(), true);
        let glo_observables = glo_observables.unwrap();
        let mut index = 0;
        for code in vec![
            "C1C", "C1P", "C2C", "C2P", "C3Q", "L1C", "L1P", "L2C", "L2P", "L3Q",
        ] {
            assert_eq!(glo_observables[index], Observable::from_str(code).unwrap());
            index += 1
        }

        //////////////////////////
        // Record testbench
        //////////////////////////
        let record = rinex.record.as_obs();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        // EPOCH[1]
        let epoch = Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 00, 00);
        let epoch = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);

        // EPOCH[2]
        let epoch = Epoch::from_gregorian_utc(2022, 06, 08, 10, 00, 30, 00);
        let epoch = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);

        // EPOCH[3]
        let epoch = Epoch::from_gregorian_utc(2020, 6, 8, 10, 1, 0, 00);
        let epoch = record.get(&(epoch, EpochFlag::Ok));
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 47);
    }
    #[test]
    fn v2_kosg0010_95o() {
        let rnx = Rinex::from_file("../test_resources/OBS/V2/KOSG0010.95O").unwrap();
        let expected: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(1995, 01, 01, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(1995, 01, 01, 11, 00, 00, 00),
            Epoch::from_gregorian_utc(1995, 01, 01, 20, 44, 30, 00),
        ];
        assert!(
            rnx.epoch().collect::<Vec<Epoch>>() == expected,
            "parsed wrong epoch content"
        );
    }
    #[test]
    fn v2_ajac3550() {
        let rnx = Rinex::from_file("../test_resources/OBS/V2/AJAC3550.21O").unwrap();
        let epochs: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2021, 12, 21, 0, 0, 0, 0),
            Epoch::from_gregorian_utc(2021, 12, 21, 0, 0, 30, 0),
        ];

        assert!(
            rnx.epoch().collect::<Vec<Epoch>>() == epochs,
            "parsed wrong epoch content"
        );

        let phase_l1c: Vec<_> = rnx
            .carrier_phase()
            .filter_map(|(e, sv, obs, value)| {
                if *obs == observable!("L1C") {
                    Some((e, sv, value))
                } else {
                    None
                }
            })
            .collect();

        for ((epoch, flag), sv, l1c) in phase_l1c {
            assert!(flag.is_ok(), "faulty epoch flag");
            if epoch == Epoch::from_str("2021-12-12T00:00:30 UTC").unwrap() {
                if sv == sv!("G07") {
                    assert_eq!(l1c, 131869667.223, "wrong L1C phase data");
                } else if sv == sv!("E31") {
                    assert_eq!(l1c, 108313833.964, "wrong L1C phase data");
                } else if sv == sv!("E33") {
                    assert_eq!(l1c, 106256338.827, "wrong L1C phase data");
                } else if sv == sv!("S23") {
                    assert_eq!(l1c, 200051837.090, "wrong L1C phase data");
                } else if sv == sv!("S36") {
                    assert_eq!(l1c, 197948874.430, "wrong L1C phase data");
                }
            } else if epoch == Epoch::from_str("2021-12-21T21:00:30 UTC").unwrap() {
                if sv == sv!("G07") {
                    assert_eq!(l1c, 131869667.223, "wrong L1C phase data");
                } else if sv == sv!("E31") {
                    assert_eq!(l1c, 108385729.352, "wrong L1C phase data");
                } else if sv == sv!("E33") {
                    assert_eq!(l1c, 106305408.320, "wrong L1C phase data");
                } else if sv == sv!("S23") {
                    assert_eq!(l1c, 200051746.696, "wrong L1C phase data");
                } else if sv == sv!("S36") {
                    assert_eq!(l1c, 197948914.912, "wrong L1C phase data");
                }
            }
        }

        let c1: Vec<_> = rnx
            .pseudo_range()
            .filter_map(|(e, sv, obs, value)| {
                if *obs == observable!("C1") {
                    Some((e, sv, value))
                } else {
                    None
                }
            })
            .collect();

        for ((epoch, flag), sv, l1c) in c1 {
            assert!(flag.is_ok(), "faulty epoch flag");
            if epoch == Epoch::from_str("2021-12-12T00:00:30 UTC").unwrap() {
                if sv == sv!("G07") {
                    assert_eq!(l1c, 25091572.300, "wrong C1 PR data");
                } else if sv == sv!("E31") {
                    assert_eq!(l1c, 25340551.060, "wrong C1 PR data");
                } else if sv == sv!("E33") {
                    assert_eq!(l1c, 27077081.020, "wrong C1 PR data");
                } else if sv == sv!("S23") {
                    assert_eq!(l1c, 38068603.000, "wrong C1 PR data");
                } else if sv == sv!("S36") {
                    assert_eq!(l1c, 37668418.660, "wrong C1 PR data");
                }
            } else if epoch == Epoch::from_str("2021-12-21T21:00:30 UTC").unwrap() {
                if sv == sv!("G07") {
                    assert_eq!(l1c, 25093963.200, "wrong C1 PR data");
                } else if sv == sv!("E31") {
                    assert_eq!(l1c, 27619715.620, "wrong C1 PR data");
                } else if sv == sv!("E33") {
                    assert_eq!(l1c, 27089585.300, "wrong C1 PR data");
                } else if sv == sv!("S23") {
                    assert_eq!(l1c, 38068585.920, "wrong C1 PR data");
                } else if sv == sv!("S36") {
                    assert_eq!(l1c, 37668426.040, "wrong C1 PR data");
                }
            }
        }

        let record = rnx.record.as_obs().unwrap();

        let epoch = epochs.get(0).unwrap();
        let flag = EpochFlag::Ok;
        let (clk_offset, vehicles) = record.get(&(*epoch, flag)).unwrap();
        assert!(clk_offset.is_none());
        assert_eq!(vehicles.len(), 26);

        let g07 = Sv::new(Constellation::GPS, 07);
        let observations = vehicles.get(&g07).unwrap();
        let mut codes: Vec<Observable> = observations.keys().map(|k| k.clone()).collect();
        codes.sort();

        let mut expected: Vec<Observable> = "L1 L2 C1 P2 D1 D2 S1 S2"
            .split_ascii_whitespace()
            .map(|k| Observable::from_str(k).unwrap())
            .collect();
        expected.sort();
        assert_eq!(codes, expected);

        let s1 = observations
            .get(&Observable::from_str("S1").unwrap())
            .unwrap();
        assert_eq!(s1.obs, 37.350);
        let s2 = observations
            .get(&Observable::from_str("S2").unwrap())
            .unwrap();
        assert_eq!(s2.obs, 35.300);

        let r04 = Sv::new(Constellation::Glonass, 04);
        let observations = vehicles.get(&r04).unwrap();

        let mut codes: Vec<Observable> = observations.keys().map(|k| k.clone()).collect();
        codes.sort();

        let mut expected: Vec<Observable> = "L1 L2 C1 C2 P2 D1 D2 S1 S2"
            .split_ascii_whitespace()
            .map(|k| Observable::from_str(k).unwrap())
            .collect();
        expected.sort();
        assert_eq!(codes, expected);

        let l1 = observations
            .get(&Observable::from_str("L1").unwrap())
            .unwrap();
        assert_eq!(l1.obs, 119147697.073);
        let l2 = observations
            .get(&Observable::from_str("L2").unwrap())
            .unwrap();
        assert_eq!(l2.obs, 92670417.710);
        let c1 = observations
            .get(&Observable::from_str("C1").unwrap())
            .unwrap();
        assert_eq!(c1.obs, 22249990.480);
        let c2 = observations
            .get(&Observable::from_str("C2").unwrap())
            .unwrap();
        assert_eq!(c2.obs, 22249983.480);
        let p2 = observations
            .get(&Observable::from_str("P2").unwrap())
            .unwrap();
        assert_eq!(p2.obs, 22249983.740);
        let d1 = observations
            .get(&Observable::from_str("D1").unwrap())
            .unwrap();
        assert_eq!(d1.obs, -1987.870);
        let d2 = observations
            .get(&Observable::from_str("D2").unwrap())
            .unwrap();
        assert_eq!(d2.obs, -1546.121);
        let s1 = observations
            .get(&Observable::from_str("S1").unwrap())
            .unwrap();
        assert_eq!(s1.obs, 47.300);
        let s2 = observations
            .get(&Observable::from_str("S2").unwrap())
            .unwrap();
        assert_eq!(s2.obs, 43.900);

        let epoch = epochs.get(1).unwrap();
        let flag = EpochFlag::Ok;
        let (clk_offset, vehicles) = record.get(&(*epoch, flag)).unwrap();
        assert!(clk_offset.is_none());
        assert_eq!(vehicles.len(), 26);

        let r04 = Sv::new(Constellation::Glonass, 04);
        let observations = vehicles.get(&r04).unwrap();
        let mut codes: Vec<Observable> = observations.keys().map(|k| k.clone()).collect();
        codes.sort();

        let mut expected: Vec<Observable> = "L1 L2 C1 C2 P2 D1 D2 S1 S2"
            .split_ascii_whitespace()
            .map(|k| Observable::from_str(k).unwrap())
            .collect();
        expected.sort();
        assert_eq!(codes, expected);

        let l1 = observations
            .get(&Observable::from_str("L1").unwrap())
            .unwrap();
        assert_eq!(l1.obs, 119207683.536);
        let l2 = observations
            .get(&Observable::from_str("L2").unwrap())
            .unwrap();
        assert_eq!(l2.obs, 92717073.850);
        let c1 = observations
            .get(&Observable::from_str("C1").unwrap())
            .unwrap();
        assert_eq!(c1.obs, 22261192.380);
        let c2 = observations
            .get(&Observable::from_str("C2").unwrap())
            .unwrap();
        assert_eq!(c2.obs, 22261185.560);
        let p2 = observations
            .get(&Observable::from_str("P2").unwrap())
            .unwrap();
        assert_eq!(p2.obs, 22261185.940);
        let d1 = observations
            .get(&Observable::from_str("D1").unwrap())
            .unwrap();
        assert_eq!(d1.obs, -2011.414);
        let d2 = observations
            .get(&Observable::from_str("D2").unwrap())
            .unwrap();
        assert_eq!(d2.obs, -1564.434);
        let s1 = observations
            .get(&Observable::from_str("S1").unwrap())
            .unwrap();
        assert_eq!(s1.obs, 46.600);
        let s2 = observations
            .get(&Observable::from_str("S2").unwrap())
            .unwrap();
        assert_eq!(s2.obs, 43.650);
    }
    #[test]
    fn v3_noa10630() {
        let rnx = Rinex::from_file("../test_resources/OBS/V3/NOA10630.22O").unwrap();
        let expected: Vec<Epoch> = vec![
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 00, 00, 00),
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 00, 30, 0),
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 01, 0, 0),
            Epoch::from_gregorian_utc(2022, 03, 04, 00, 52, 30, 0),
        ];
        assert!(
            rnx.epoch().collect::<Vec<Epoch>>() == expected,
            "parsed wrong epoch content"
        );

        let record = rnx.record.as_obs().unwrap();
        for (e_index, ((_e, flag), (clk_offset, vehicles))) in record.iter().enumerate() {
            assert!(flag.is_ok());
            assert!(clk_offset.is_none());
            assert_eq!(vehicles.len(), 9);
            if e_index < 3 {
                let keys: Vec<Sv> = vehicles.keys().map(|k| *k).collect();
                let expected: Vec<Sv> = vec![
                    Sv::new(Constellation::GPS, 01),
                    Sv::new(Constellation::GPS, 03),
                    Sv::new(Constellation::GPS, 04),
                    Sv::new(Constellation::GPS, 09),
                    Sv::new(Constellation::GPS, 17),
                    Sv::new(Constellation::GPS, 19),
                    Sv::new(Constellation::GPS, 21),
                    Sv::new(Constellation::GPS, 22),
                    Sv::new(Constellation::GPS, 31),
                ];
                assert_eq!(keys, expected);
            } else {
                let keys: Vec<Sv> = vehicles.keys().map(|k| *k).collect();
                let expected: Vec<Sv> = vec![
                    Sv::new(Constellation::GPS, 01),
                    Sv::new(Constellation::GPS, 03),
                    Sv::new(Constellation::GPS, 04),
                    Sv::new(Constellation::GPS, 06),
                    Sv::new(Constellation::GPS, 09),
                    Sv::new(Constellation::GPS, 17),
                    Sv::new(Constellation::GPS, 19),
                    Sv::new(Constellation::GPS, 21),
                    Sv::new(Constellation::GPS, 31),
                ];
                assert_eq!(keys, expected);
            }
            for (sv, _observations) in vehicles {
                assert_eq!(sv.constellation, Constellation::GPS);
            }
        }
    }
    #[cfg(feature = "flate2")]
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_esbc00dnk_r_2020() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();

        /*
         * Header tb
         */
        let header = rnx.header;
        assert_eq!(header.station, "ESBC00DNK");
        assert_eq!(header.station_id, "10118M001");
        assert_eq!(header.marker_type, Some(MarkerType::Geodetic));

        /*
         * Observation specific
         */
        let obs = header.obs.as_ref();
        assert!(obs.is_some());
        let obs = obs.unwrap();

        for (k, v) in &obs.codes {
            if *k == Constellation::GPS {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C1W C2L C2W C5Q D1C D2L D2W D5Q L1C L2L L2W L5Q S1C S1W S2L S2W S5Q"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::Glonass {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C1P C2C C2P C3Q D1C D1P D2C D2P D3Q L1C L1P L2C L2P L3Q S1C S1P S2C S2P S3Q"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::BeiDou {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C2I C6I C7I D2I D6I D7I L2I L6I L7I S2I S6I S7I"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::QZSS {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C2L C5Q D1C D2L D5Q L1C L2L L5Q S1C S2L S5Q"
                        .split_ascii_whitespace()
                        .map(|k| Observable::from_str(k).unwrap())
                        .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::Galileo {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> =
                    "C1C C5Q C6C C7Q C8Q D1C D5Q D6C D7Q D8Q L1C L5Q L6C L7Q L8Q S1C S5Q S6C S7Q S8Q"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else if *k == Constellation::Geo {
                let mut sorted = v.clone();
                sorted.sort();
                let mut expected: Vec<Observable> = "C1C C5I D1C D5I L1C L5I S1C S5I"
                    .split_ascii_whitespace()
                    .map(|k| Observable::from_str(k).unwrap())
                    .collect();
                expected.sort();
                assert_eq!(sorted, expected);
            } else {
                panic!("decoded unexpected constellation");
            }
        }

        assert_eq!(header.glo_channels.len(), 23);
        let mut keys: Vec<Sv> = header.glo_channels.keys().map(|k| *k).collect();
        keys.sort();
        assert_eq!(
            vec![
                Sv::from_str("R01").unwrap(),
                Sv::from_str("R02").unwrap(),
                Sv::from_str("R03").unwrap(),
                Sv::from_str("R04").unwrap(),
                Sv::from_str("R05").unwrap(),
                Sv::from_str("R06").unwrap(),
                Sv::from_str("R07").unwrap(),
                Sv::from_str("R08").unwrap(),
                Sv::from_str("R09").unwrap(),
                Sv::from_str("R10").unwrap(),
                Sv::from_str("R11").unwrap(),
                Sv::from_str("R12").unwrap(),
                Sv::from_str("R13").unwrap(),
                Sv::from_str("R14").unwrap(),
                Sv::from_str("R15").unwrap(),
                Sv::from_str("R16").unwrap(),
                Sv::from_str("R17").unwrap(),
                Sv::from_str("R18").unwrap(),
                Sv::from_str("R19").unwrap(),
                Sv::from_str("R20").unwrap(),
                Sv::from_str("R21").unwrap(),
                Sv::from_str("R23").unwrap(),
                Sv::from_str("R24").unwrap(),
            ],
            keys
        );
        let mut values: Vec<i8> = header.glo_channels.values().map(|k| *k).collect();
        values.sort();
        assert_eq!(
            vec![
                -7_i8, -7_i8, -4_i8, -4_i8, -3_i8, -2_i8, -2_i8, -1_i8, -1_i8, 0_i8, 0_i8, 1_i8,
                1_i8, 2_i8, 2_i8, 3_i8, 3_i8, 4_i8, 4_i8, 5_i8, 5_i8, 6_i8, 6_i8
            ],
            values
        );
    }
    #[cfg(feature = "flate2")]
    #[test]
    #[cfg(feature = "flate2")]
    fn v3_mojn00dnk_r_2020() {
        let rnx =
            Rinex::from_file("../test_resources/CRNX/V3/MOJN00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();
        /*
         * Test IRNSS vehicles
         */
        let mut irnss_sv: Vec<Sv> = rnx
            .sv()
            .filter_map(|sv| {
                if (sv.constellation == Constellation::IRNSS) {
                    Some(sv)
                } else {
                    None
                }
            })
            .collect();
        irnss_sv.sort();

        assert_eq!(
            irnss_sv,
            vec![
                sv!("I01"),
                sv!("I02"),
                sv!("I04"),
                sv!("I05"),
                sv!("I06"),
                sv!("I09")
            ],
            "IRNSS sv badly identified"
        );
    }
    /*
        #[test]
        fn obs_v3_duth0630_processing() {
            let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
                .unwrap();
            let record = rinex.record.as_obs()
                .unwrap();

            // MIN
            let min = record.min();
            let g01 = min.get(&Sv::from_str("G01").unwrap()).unwrap();
            let s1c = g01.get(&Observable::from_str("S1C").unwrap()).unwrap();
            assert_eq!(*s1c, 49.5);

            // MAX
            let max = record.max();
            let g01 = max.get(&Sv::from_str("G01").unwrap()).unwrap();
            let s1c = g01.get(&Observable::from_str("S1C").unwrap()).unwrap();
            assert_eq!(*s1c, 51.250);

            // MEAN
            let mean = record.mean();
            let g01 = mean.get(&Sv::from_str("G01").unwrap()).unwrap();
            let s1c = g01.get(&Observable::from_str("S1C").unwrap()).unwrap();
            assert_eq!(*s1c, (51.250 + 50.750 + 49.5)/3.0);

            let g06 = mean.get(&Sv::from_str("G06").unwrap()).unwrap();
            let s1c = g06.get(&Observable::from_str("S1C").unwrap()).unwrap();
            assert_eq!(*s1c, 43.0);

            // STDVAR
            let stdvar = record.stdvar();
            let mean = (51.25_f64 + 50.75_f64 + 49.5_f64)/3.0_f64;
            let expected = ((51.25_f64 - mean).powf(2.0_f64) + (50.75_f64 - mean).powf(2.0_f64) + (49.5_f64 - mean).powf(2.0_f64)) / 3.0f64;
            let g01 = stdvar.get(&Sv::from_str("G01").unwrap()).unwrap();
            let s1c =  g01.get(&Observable::from_str("S1C").unwrap()).unwrap();
            assert_eq!(*s1c, expected);
        }
        fn test_combinations(combinations: Vec<(Observable, Observable)>, signals: Vec<Observable>) {
            /*
             * test nb of combinations
             */
            let mut nb_pr = 0;
            let mut nb_ph = 0;
            for sig in signals {
                let code = sig.code();
                if sig.is_phase_observable() {
                    nb_pr += 1;
                }
                if sig.is_pseudorange_observable() {
                    nb_ph += 1;
                }
            }
            assert_eq!(combinations.len(), nb_pr-1+nb_ph-1, "Wrong number of combinations, expecting {} | got: {:?}", nb_pr+nb_ph-2, combinations);
            /*
             * test formed combinations
             * (M > 1) => 1
             * 1       => 2
             */
            for (lhs, reference) in combinations {
                let lhs_code = lhs.to_string();
                let reference_code = reference.to_string();
                let lhs_carrier = &lhs_code[1..2];
                let reference_carrier = &reference_code[1..2];
                if lhs_carrier != "1" {
                    assert_eq!(reference_carrier, "1");
                } else {
                    assert_eq!(reference_carrier, "2");
                }
            }
        }
        #[test]
        fn obs_v2_aopr0010_17o() {
            let rinex = Rinex::from_file("../test_resources/OBS/V2/aopr0010.17o")
                .unwrap();
            let record = rinex.record.as_obs()
                .unwrap();
            let mut signals = vec![
                Observable::from_str("L1").unwrap(),
                Observable::from_str("L2").unwrap(),
                Observable::from_str("C1").unwrap(),
                Observable::from_str("P1").unwrap(),
                Observable::from_str("P2").unwrap(),
            ];
            for combination in [
                Combination::GeometryFree,
                Combination::NarrowLane,
                Combination::WideLane,
                Combination::MelbourneWubbena,
            ] {
                let combined = record.combine(combination);
                let mut combinations: Vec<(Observable, Observable)> =
                    combined.keys().map(|(lhs, rhs)| (lhs.clone(), rhs.clone())).collect();
                test_combinations(combinations, signals.clone());
            }
            /*
             * Iono Delay Detector
             */
            let dt = rinex.sampling_interval().unwrap();
            let ionod = record.iono_delay_detector(dt);
        }
        #[test]
        fn obs_v3_duth0630_gnss_combinations() {
            let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
                .unwrap();
            let record = rinex.record.as_obs()
                .unwrap();
            let mut signals = vec![
                Observable::from_str("C1C").unwrap(),
                Observable::from_str("C2W").unwrap(),
                Observable::from_str("C2P").unwrap(),
                Observable::from_str("L1C").unwrap(),
                Observable::from_str("L2P").unwrap(),
                Observable::from_str("L2W").unwrap(),
            ];
            for combination in [
                Combination::GeometryFree,
                Combination::NarrowLane,
                Combination::WideLane,
                Combination::MelbourneWubbena,
            ] {
                let combined = record.combine(combination);
                let mut combinations: Vec<(Observable, Observable)> =
                    combined.keys().map(|(lhs, rhs)| (lhs.clone(), rhs.clone())).collect();
                test_combinations(combinations, signals.clone());
            }
            /*
             * Iono Delay Detector
             */
            let dt = rinex.sampling_interval().unwrap();
            let ionod = record.iono_delay_detector(dt);
        }
        #[test]
        fn obs_v3_esbcd00dnk_r_2020_gnss_combinations() {
            let rinex = Rinex::from_file("../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz")
                .unwrap();
            let record = rinex.record.as_obs()
                .unwrap();
            let gf = record.combine(Combination::GeometryFree);
            let mut combinations: Vec<(Observable, Observable)> =
                gf.keys().map(|(lhs, rhs)| (lhs.clone(), rhs.clone())).collect();
            let mut signals = vec![
                Observable::from_str("C1C").unwrap(),
                Observable::from_str("C1W").unwrap(),
                Observable::from_str("C2I").unwrap(),
                Observable::from_str("C2L").unwrap(),
                Observable::from_str("C2W").unwrap(),
                Observable::from_str("C5I").unwrap(),
                Observable::from_str("C5Q").unwrap(),
                Observable::from_str("C6C").unwrap(),
                Observable::from_str("C6I").unwrap(),
                Observable::from_str("C7I").unwrap(),
                Observable::from_str("C7Q").unwrap(),
                Observable::from_str("C8Q").unwrap(),

                Observable::from_str("L1C").unwrap(),
                Observable::from_str("L2I").unwrap(),
                Observable::from_str("L2L").unwrap(),
                Observable::from_str("L3Q").unwrap(),
                Observable::from_str("L2W").unwrap(),
                Observable::from_str("L5I").unwrap(),
                Observable::from_str("L5Q").unwrap(),
                Observable::from_str("L6C").unwrap(),
                Observable::from_str("L6I").unwrap(),
                Observable::from_str("L7I").unwrap(),
                Observable::from_str("L7Q").unwrap(),
                Observable::from_str("L8Q").unwrap(),
            ];
            test_combinations(combinations, signals);
        }
    */
}
