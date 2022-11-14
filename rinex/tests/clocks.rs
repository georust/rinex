#[cfg(test)]
mod test {
    use rinex::prelude::*;
    use rinex::epoch;
    use rinex::clocks;
    use std::str::FromStr;
    use rinex::clocks::record::{DataType, System};
    #[test]
    fn v3_usno_example() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CLK/V3/USNO1.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_clocks_rinex(), true);
        assert_eq!(rinex.header.clocks.is_some(), true);
        let clocks = rinex.header.clocks
            .as_ref()
            .unwrap();
        assert_eq!(clocks.codes, vec![
            DataType::AS,
            DataType::AR,
            DataType::CR,
            DataType::DR]);
        assert_eq!(clocks.agency, Some(clocks::Agency {
            code: String::from("USN"),
            name: String::from("USNO USING GIPSY/OASIS-II"),
        }));
        assert_eq!(clocks.station, Some(clocks::Station {
            name: String::from("USNO"),
            id: String::from("40451S003"),
        }));
        assert_eq!(rinex.epochs().len(), 1);
        let record = rinex.record.as_clock();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        for (e, data_types) in record.iter() {
            assert_eq!(*e, epoch::Epoch::from_str("1994 07 14 20 59  0.000000").unwrap());
            for (data_type, systems) in data_types.iter() {
                assert_eq!(systems.len(), 1);
                if *data_type == DataType::AR {
                    for (system, data) in systems.iter() {
                        assert_eq!(*system, System::Station("AREQ".to_string()));
                        assert_eq!(data.bias,  -0.123456789012);
                        assert_eq!(data.bias_sigma, Some(-1.23456789012E+0));
                        assert_eq!(data.rate, Some(-12.3456789012));
                        assert_eq!(data.rate_sigma, Some(-123.456789012));
                    }
                } else if *data_type == DataType::AS {
                    for (system, _) in systems.iter() {
                        assert_eq!(*system, System::Sv(Sv {
                            constellation: Constellation::GPS,
                            prn: 16
                        }));
                    }

                } else if *data_type == DataType::CR {
                    for (system, _) in systems.iter() {
                        assert_eq!(*system, System::Station("USNO".to_string()));
                    }

                } else if *data_type == DataType::DR {
                    for (system, _) in systems.iter() {
                        assert_eq!(*system, System::Station("USNO".to_string()));
                    }

                } else {
                    panic!("identified unexpected data type \"{}\"", data_type);
                }
            }
        }
    }
    #[test]
    fn v3_04_example1() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CLK/V3/example1.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_clocks_rinex(), true);
        assert_eq!(rinex.header.clocks.is_some(), true);
        let clocks = rinex.header.clocks
            .as_ref()
            .unwrap();
        assert_eq!(clocks.codes, vec![DataType::AS, DataType::AR]);
        assert_eq!(clocks.agency, Some(clocks::Agency {
            code: String::from("USN"),
            name: String::from("USNO USING GIPSY/OASIS-II"),
        }));
        assert_eq!(rinex.epochs().len(), 1);
        let record = rinex.record.as_clock();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        for (e, data_types) in record.iter() {
            assert_eq!(*e, epoch::Epoch::from_str("1994 07 14 20 59  0.000000").unwrap());
            for (data_type, systems) in data_types.iter() {
                if *data_type == DataType::AR {
                    assert_eq!(systems.len(), 4);
                    for (system, data) in systems.iter() {
                        let areq_usa = System::Station("AREQ00USA".to_string());
                        let gold = System::Station("GOLD".to_string());
                        let tidb = System::Station("TIDB".to_string());
                        let hark = System::Station("HARK".to_string());
                        if *system == areq_usa {
                            assert_eq!(data.bias,  -0.123456789012);
                            assert_eq!(data.bias_sigma, Some( -0.123456789012E+01));
                            assert_eq!(data.rate, Some(-0.123456789012E+02));
                            assert_eq!(data.rate_sigma, Some( -0.123456789012E+03));
                        } else if *system == gold { 
                            assert_eq!(data.bias, -0.123456789012E-01);
                            assert_eq!(data.bias_sigma, Some( -0.123456789012E-02));
                            assert_eq!(data.rate, Some(-0.123456789012E-03));
                            assert_eq!(data.rate_sigma, Some(-0.123456789012E-04));
                        } else if *system == tidb { 
                            assert_eq!(data.bias,  0.123456789012E+00);
                            assert_eq!(data.bias_sigma, Some( 0.123456789012E+00));
                        } else if *system == hark { 
                            assert_eq!(data.bias, 0.123456789012E+00);
                            assert_eq!(data.bias_sigma, Some( 0.123456789012E+00));
                        } else {
                            panic!("falsely identified system \"{}\"", *system);
                        }
                    }
                } else if *data_type == DataType::AS {
                    assert_eq!(systems.len(), 1);
                    for (system, data) in systems.iter() {
                        assert_eq!(*system, System::Sv(Sv {
                            constellation: Constellation::GPS,
                            prn: 16
                        }));
                        assert_eq!(data.bias, -0.123456789012E+00);
                        assert_eq!(data.bias_sigma, Some( -0.123456789012E-01));
                    }

                } else {
                    panic!("identified unexpected data type \"{}\"", data_type);
                }
            }
        }
    }
    #[test]
    fn v3_04_example2() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/CLK/V3/example2.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_clocks_rinex(), true);
        assert_eq!(rinex.header.clocks.is_some(), true);
        let clocks = rinex.header.clocks
            .as_ref()
            .unwrap();
        assert_eq!(clocks.codes, vec![DataType::AR, DataType::AS]);
        assert_eq!(clocks.agency, Some(clocks::Agency {
            code: String::from("IGS"),
            name: String::from("IGSACC @ GA and MIT"),
        }));
        assert_eq!(rinex.epochs().len(), 1);
        let record = rinex.record.as_clock();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        /*for (e, data_types) in record.iter() {
            assert_eq!(*e, epoch::Epoch {
                date: epoch::str2date("2017 03 11 00 00  0.000000").unwrap(),
                flag: epoch::EpochFlag::Ok,
            });
            for (data_type, systems) in data_types.iter() {
                if *data_type == DataType::AR {
                    assert_eq!(systems.len(), 4);
                } else if *data_type == DataType::AS {
                    assert_eq!(systems.len(), 2);
                } else {
                    panic!("identified unexpected data type \"{}\"", data_type);
                }
            }
        }*/
    }
}
