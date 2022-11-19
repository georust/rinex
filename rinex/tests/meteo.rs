#[cfg(test)]
mod test {
    use std::str::FromStr;
    use rinex::meteo::*;
    use rinex::prelude::*;
    #[test]
    fn v2_abvi0010_15m() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/MET/V2/abvi0010.15m";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_meteo_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_some(), true);
        
        let record = rinex.record.as_meteo();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 74);

        let epoch = Epoch::from_gregorian_utc(2015, 01, 01, 00, 00, 00, 00);
        let content = record.get(&epoch);
        assert_eq!(content.is_some(), true);
        let content = content.unwrap();
        
        let epoch = Epoch::from_gregorian_utc(2015, 01, 01, 9, 00, 00, 00);
        let content = record.get(&epoch);
        assert_eq!(content.is_some(), true);
        let content = content.unwrap();
        
        let epoch = Epoch::from_gregorian_utc(2015, 01, 01, 19, 25, 0, 0);
        let content = record.get(&epoch);
        assert_eq!(content.is_some(), true);
        let content = content.unwrap();
    }
    #[test]
    fn v4_example1() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/MET/V4/example1.txt";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_meteo_rinex(), true);
        assert_eq!(rinex.header.obs.is_none(), true);
        assert_eq!(rinex.header.meteo.is_some(), true);
        let record = rinex.record.as_meteo();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        assert_eq!(record.len(), 5);

        // test epoch content
        for (_, obs) in record.iter() {
            for (obs, data) in obs.iter() {
                if *obs == Observable::Pressure {
                    assert_eq!(*data, 993.3);
                } else if *obs == Observable::HumidityRate {
                    assert_eq!(*data, 90.0);
                }
            }
        }
        let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 00, 00, 00, 00);
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == Observable::Temperature {
                assert_eq!(*data, 23.0);
            }
        }
        let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 0, 30, 0);
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == Observable::Temperature {
                assert_eq!(*data, 23.0);
            }
        }
        let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 1, 0, 00);
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
        let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 1, 30, 0);
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
        let epoch = Epoch::from_gregorian_utc(2021, 1, 7, 0, 2, 0, 00);
        let e = record.get(&epoch).unwrap();
        for (obs, data) in e.iter() {
            if *obs == Observable::Temperature {
                assert_eq!(*data, 23.1);
            }
        }
    }
}
