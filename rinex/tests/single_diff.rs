#[cfg(test)]
mod test {
    use rinex::*;
    #[test]
    fn test_diff_null() {
        let pool = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path = pool.to_owned() + "OBS/V2/aopr0010.17o";
        let mut rnx = Rinex::from_file(&path)
            .unwrap();
        // run tb
        let record = rnx.record
            .as_obs()
            .unwrap();
        // we initially got phase data
        for (_, (_, svs)) in record.iter() {
            for (_, observables) in svs.iter() {
                for (observable, data) in observables.iter() {
                    if is_phase_carrier_obs_code!(observable) {
                        assert_eq!(data.obs != 0.0, true);
                    }
                }
            }
        }
        assert_eq!(rnx.observation_diff_mut(&rnx.clone()).is_ok(), true);
        // all phase data cancelled
        let record = rnx.record
            .as_obs()
            .unwrap();
        for (_, (_, svs)) in record.iter() {
            for (_, observables) in svs.iter() {
                for (observable, data) in observables.iter() {
                    if is_phase_carrier_obs_code!(observable) {
                        assert_eq!(data.obs, 0.0);
                    }
                }
            }
        }
    }
    #[test]
    fn test_diff_failure() {
        let pool = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let path = pool.to_owned() + "NAV/V2/amel0010.21g";
        let mut rnx = Rinex::from_file(&path)
            .unwrap();
        assert_eq!(rnx.observation_diff_mut(&rnx.clone()).is_err(), true);
    }
    #[test]
    fn test_diff() {
        let pool = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/";
        let file_a = pool.to_owned() + "OBS/V3/LARM0010.22O";
        let file_b = pool.to_owned() + "OBS/V3/VLNS0010.22O";
        // parse A
        let rnx_a = Rinex::from_file(&file_a)
            .unwrap();
        let rec_a = rnx_a.record
            .as_obs()
            .unwrap();
        // parse B
        let rnx_b = Rinex::from_file(&file_b)
            .unwrap();
        let rec_b = rnx_b.record
            .as_obs()
            .unwrap();
        // process (a-b)
        let rnx = rnx_a.observation_diff(&rnx_b);
        assert_eq!(rnx.is_ok(), true); //tb
        let rnx = rnx.unwrap();
        let rec = rnx.record
            .as_obs()
            .unwrap();

        // run tb
        // epoch1: 2022/01/01 midnight
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 01 01 00 00 0.0")
                .unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        //  rnx_a :
        //      GPS [01, 08, 10, x   16, 18, 21, 23, 26, 27, 32 ]
        //      GLO [01, 02, 07, 08, 14, 15, 17, 22, 23, 24]

        //  rnx_b : 
        //      GPS [ x  08, 10, 15, 16, 18, 21, 23, x   27, 32 ]
        //      GLO [01, x   07, 08, 14, 15, 17, 22, 23, 24] 
        let e = rec.get(&epoch);
        assert_eq!(e.is_some(), true); // A/B share this Epoch 

        // browse rnx
        for (e, (_, vehicules)) in rec.iter() {
            let (_, vehicules_a) = rec_a.get(e)
                .unwrap();
            let (_, vehicules_b) = rec_b.get(e)
                .unwrap();
            for (sv, observables) in vehicules.iter() {
                let data_a = vehicules_a.get(sv);
                assert_eq!(data_a.is_some(), true); // shared vehicules
                let observables_a = data_a.unwrap();
                let data_b = vehicules_b.get(sv);
                assert_eq!(data_b.is_some(), true); // shared vehicules
                let observables_b = data_b.unwrap();
                for (obscode, obsdata) in observables.iter() {
                    if is_phase_carrier_obs_code!(obscode) {
                        // test actuall (A)-(B) ops
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did not produce unexpected data
                        let data_a = data_a.unwrap();
                        let data_b = observables_b.get(obscode);
                        assert_eq!(data_b.is_some(), true); // we did not produce unexpected data
                        let data_b = data_b.unwrap();
                        // test actual (A)-(B)
                        assert_eq!(obsdata.obs == (data_a.obs - data_b.obs), true);
                    } else {
                        // this is not a phase observable
                        // left untouched (data is preserved)
                        // test whether this is true or not
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did preserve 
                        let data_a = data_a.unwrap();
                        assert_eq!(obsdata == data_a, true); // we did preserve correctly
                    }
                }
            }
        }

        // epoch2: epoch1 + 30'
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 01 01 00 00 30.0")
                .unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        //  rnx_a:
        //      GPS [01, 08, 10, x   16, 18, 21, 23, 26, 27, 32] 
        //      GLO [01, 02, 07, 08, 14, 15, 17, 22, 23, 24] 
        //  rnx_b:
        //      GPS [x   08, 10, 15, 16, 18, 21, 23, x   27, 32]
        //      GLO [01, x   07, 08, 14, x   17, 22, 23, 24]
        let e = rec.get(&epoch);
        assert_eq!(e.is_some(), true); // A/B share this Epoch 
        
        // browse rnx
        for (e, (_, vehicules)) in rec.iter() {
            let (_, vehicules_a) = rec_a.get(e)
                .unwrap();
            let (_, vehicules_b) = rec_b.get(e)
                .unwrap();
            for (sv, observables) in vehicules.iter() {
                let data_a = vehicules_a.get(sv);
                assert_eq!(data_a.is_some(), true); // shared vehicules
                let observables_a = data_a.unwrap();
                let data_b = vehicules_b.get(sv);
                assert_eq!(data_b.is_some(), true); // shared vehicules
                let observables_b = data_b.unwrap();
                for (obscode, obsdata) in observables.iter() {
                    if is_phase_carrier_obs_code!(obscode) {
                        // test actuall (A)-(B) ops
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did not produce unexpected data
                        let data_a = data_a.unwrap();
                        let data_b = observables_b.get(obscode);
                        assert_eq!(data_b.is_some(), true); // we did not produce unexpected data
                        let data_b = data_b.unwrap();
                        // test actual (A)-(B)
                        assert_eq!(obsdata.obs == (data_a.obs - data_b.obs), true);
                    } else {
                        // this is not a phase observable
                        // left untouched (data is preserved)
                        // test whether this is true or not
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did preserve 
                        let data_a = data_a.unwrap();
                        assert_eq!(obsdata == data_a, true); // we did preserve correctly
                    }
                }
            }
        }

        // epoch3: epoch2 + 30'
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 01 01 00 01 00.0")
                .unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        //  rnx_a:
        //      GPS [01, 08, 10, x   16, 18, 21, 23, 26, 27, 32]
        //      GLO [01, 02, 07, 08, 14, 15, 17, 22, 23, 24]
        //  rnx_b:
        //      GPS [x   08, 10, 15, 16, 18, 21, 23, 27, 32]
        //      GLO [01, x   07, 08, 14, 15, 17, 22, 23, 24]
        let e = rec.get(&epoch);
        assert_eq!(e.is_some(), true); // A/B share this Epoch 

        // browse rnx
        for (e, (_, vehicules)) in rec.iter() {
            let (_, vehicules_a) = rec_a.get(e)
                .unwrap();
            let (_, vehicules_b) = rec_b.get(e)
                .unwrap();
            for (sv, observables) in vehicules.iter() {
                let data_a = vehicules_a.get(sv);
                assert_eq!(data_a.is_some(), true); // shared vehicules
                let observables_a = data_a.unwrap();
                let data_b = vehicules_b.get(sv);
                assert_eq!(data_b.is_some(), true); // shared vehicules
                let observables_b = data_b.unwrap();
                for (obscode, obsdata) in observables.iter() {
                    if is_phase_carrier_obs_code!(obscode) {
                        // test actuall (A)-(B) ops
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did not produce unexpected data
                        let data_a = data_a.unwrap();
                        let data_b = observables_b.get(obscode);
                        assert_eq!(data_b.is_some(), true); // we did not produce unexpected data
                        let data_b = data_b.unwrap();
                        // test actual (A)-(B)
                        assert_eq!(obsdata.obs == (data_a.obs - data_b.obs), true);
                    } else {
                        // this is not a phase observable
                        // left untouched (data is preserved)
                        // test whether this is true or not
                        let data_a = observables_a.get(obscode);
                        assert_eq!(data_a.is_some(), true); // we did preserve 
                        let data_a = data_a.unwrap();
                        assert_eq!(obsdata == data_a, true); // we did preserve correctly
                    }
                }
            }
        }

        // epoch4: epoch3 +30'
        // only exists in RNX(A) (for testing purposes)
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 01 01 00 01 30.0")
                .unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        let e = rec.get(&epoch);
        assert_eq!(e.is_none(), true); // A/B do not share this Epoch 

        // Test the (-) operator !
        let rnxx = rnx_a.clone() - rnx_b.clone();
        let recc = rnxx.record
            .as_obs()
            .unwrap();
        assert_eq!(rec, recc);

        // Test the (-=) operator !
        let mut rnx_a = rnx_a.clone();
        rnx_a -= rnx_b.clone();
        let recc = rnx_a.record
            .as_obs()
            .unwrap();
        assert_eq!(rec, recc);
    }
}
