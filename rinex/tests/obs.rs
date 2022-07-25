#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::constellation::Constellation;
    #[test]
    fn v2_aopr0010_17o() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/aopr0010.17o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_observation_rinex(), true);
        assert_eq!(rinex.header.obs.is_some(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);
    }
    #[test]
    fn v4_kms300dnk_r_2022_v3crx() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
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
        for code in vec!["C1C","C1P","C2C","C2P","C3Q","L1C","L1P","L2C","L2P","L3Q"] {
            assert_eq!(glo_observables[index], code);
            index += 1
        }
        
        //////////////////////////
        // Record testbench
        //////////////////////////
        let record = rinex.record.as_obs();
        assert_eq!(record.is_some(), true);
        let record = record.unwrap();
        // EPOCH[1]
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 06 08 10 00 00.0000000").unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);
        
        // EPOCH[2]
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 06 08 10 00 30.0000000").unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);
        
        // EPOCH[3]
        let epoch = epoch::Epoch {
            date: epoch::str2date("2022 06 08 10 01 00.0000000").unwrap(),
            flag: epoch::EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 47);
    }
}
