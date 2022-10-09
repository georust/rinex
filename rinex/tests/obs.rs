#[cfg(test)]
mod test {
    use rinex::*;
    use rinex::constellation::Constellation;
	use rinex::observation::{LliFlags, Ssi};
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

		let obs_hd = rinex.header.obs.as_ref().unwrap();
		let record = rinex.record.as_obs();
		assert_eq!(record.is_some(), true);
		let record = record.unwrap();

		///////////////////////////
		// This file is GPS only
		///////////////////////////
		let obscodes = obs_hd.codes.get(&Constellation::GPS);
		assert_eq!(obscodes.is_some(), true);
		let obscodes = obscodes.unwrap();
		assert_eq!(obscodes, &vec![
			String::from("L1"),
			String::from("L2"),
			String::from("C1"),
			String::from("P1"),
			String::from("P2")]);
		
		// test epoch [1]
		let epoch = epoch::Epoch {
			date: epoch::str2date("2017 01 01 0 0 0.0").unwrap(),
			flag: epoch::EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
		assert_eq!(epoch.is_some(), true);
		let (clk_offset, epoch) = epoch.unwrap();
		assert_eq!(clk_offset.is_none(), true);
		assert_eq!(epoch.len(), 10);

		// G31
		let sv = Sv {
			constellation: Constellation::GPS,
			prn: 31,
		};
		let observations = epoch.get(&sv);
		assert_eq!(observations.is_some(), true);
		let observations = observations.unwrap();

		// L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, -14746974.730);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz54));
		// L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, -11440396.209);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz48_53));
		// C1
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 22513484.637);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		// P1
		let observed = observations.get(&String::from("P1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 22513484.772);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		// P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 22513487.370);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 

        //G26
		let sv = Sv {
			constellation: Constellation::GPS,
			prn: 26,
		};
		let observations = epoch.get(&sv);
		assert_eq!(observations.is_some(), true);
		let observations = observations.unwrap();

		// L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, -15834397.660);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz54));
		// L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, -12290568.980);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz54));
		// C1
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 21540206.165);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		// P1
		let observed = observations.get(&String::from("P1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 21540206.156);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		// P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 21540211.941);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 

		// test epoch [2]
		let epoch = epoch::Epoch {
			date: epoch::str2date("2017 01 01 3 33 40.0").unwrap(),
			flag: epoch::EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
		assert_eq!(epoch.is_some(), true);
		let (clk_offset, epoch) = epoch.unwrap();
		assert_eq!(clk_offset.is_none(), true);
		assert_eq!(epoch.len(), 9);
		
        // G30
		let sv = Sv {
			constellation: Constellation::GPS,
			prn: 30,
		};
		let observations = epoch.get(&sv);
		assert_eq!(observations.is_some(), true);
		let observations = observations.unwrap();

		// L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, -4980733.185); 
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz48_53));
		// L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, -3805623.873);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi, Some(Ssi::DbHz42_47));
		// C1
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24352349.168);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		// P1
		let observed = observations.get(&String::from("P1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24352347.924);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
        // P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24352356.156);
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING));
		assert_eq!(observed.ssi.is_none(), true); 
		
		// test epoch [3]
		let epoch = epoch::Epoch {
			date: epoch::str2date("2017 01 01 6 9 10.0").unwrap(),
			flag: epoch::EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
		assert_eq!(epoch.is_some(), true);
		let (clk_offset, epoch) = epoch.unwrap();
		assert_eq!(clk_offset.is_none(), true);
		assert_eq!(epoch.len(), 11);
    }
	#[test]
	fn v2_npaz3550_210() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/npaz3550.21o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_observation_rinex(), true);
        assert_eq!(rinex.header.obs.is_some(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);

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
		assert_eq!(obscodes, &vec![
			String::from("C1"),
			String::from("L1"),
			String::from("L2"),
			String::from("P2"),
			String::from("S1"),
			String::from("S2")]);
		let obscodes = obs_hd.codes.get(&Constellation::Glonass);
		assert_eq!(obscodes.is_some(), true);
		let obscodes = obscodes.unwrap();
		assert_eq!(obscodes, &vec![
			String::from("C1"),
			String::from("L1"),
			String::from("L2"),
			String::from("P2"),
			String::from("S1"),
			String::from("S2")]);
		
		// test epoch [1]
		let epoch = epoch::Epoch {
			date: epoch::str2date("2021 12 21 0 0 0.0").unwrap(),
			flag: epoch::EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
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
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 22288985.512); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		// L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 117129399.048);
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN)); 
		assert_eq!(observed.ssi, Some(Ssi::DbHz36_41));
		// L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, 91269672.416);  
		assert_eq!(observed.lli, Some(LliFlags::UNDER_ANTI_SPOOFING)); 
		assert_eq!(observed.ssi, Some(Ssi::DbHz36_41));
		// P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, 22288987.972);        
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		// S1
		let observed = observations.get(&String::from("S1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, 44.000);
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		// S2
		let observed = observations.get(&String::from("S2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, 27.000);
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 

        //R19
		let sv = Sv {
			constellation: Constellation::Glonass,
			prn: 19,
		};
		let observations = epoch.get(&sv);
		assert_eq!(observations.is_some(), true);
		let observations = observations.unwrap();

		// C1
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 23250776.648);
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		// L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 124375967.254); 
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN)); 
		assert_eq!(observed.ssi, Some(Ssi::DbHz0));
		// L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_none(), true);
		// P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_none(), true);
		// S1
		let observed = observations.get(&String::from("S1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
        assert_eq!(observed.obs, 32.000);
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		// S2
		let observed = observations.get(&String::from("S2"));
		assert_eq!(observed.is_none(), true);
	}
	#[test]
	fn v2_rovn0010_210() {
        let test_resource = 
            env!("CARGO_MANIFEST_DIR").to_owned() 
            + "/../test_resources/OBS/V2/rovn0010.21o";
        let rinex = Rinex::from_file(&test_resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex.unwrap();
        assert_eq!(rinex.is_observation_rinex(), true);
        assert_eq!(rinex.header.obs.is_some(), true);
        assert_eq!(rinex.header.meteo.is_none(), true);

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
		assert_eq!(obscodes, &vec![
			String::from("C1"),
			String::from("C2"),
			String::from("C5"),
			String::from("L1"),
			String::from("L2"),
			String::from("L5"),
			String::from("P1"),
			String::from("P2"),
			String::from("S1"),
			String::from("S2"),
			String::from("S5")]);
		
		let obscodes = obs_hd.codes.get(&Constellation::Glonass);
		assert_eq!(obscodes.is_some(), true);
		let obscodes = obscodes.unwrap();
		assert_eq!(obscodes, &vec![
			String::from("C1"),
			String::from("C2"),
			String::from("C5"),
			String::from("L1"),
			String::from("L2"),
			String::from("L5"),
			String::from("P1"),
			String::from("P2"),
			String::from("S1"),
			String::from("S2"),
			String::from("S5")]);
		
		// test epoch [1]
		let epoch = epoch::Epoch {
			date: epoch::str2date("2021 01 01 0 0 0.0").unwrap(),
			flag: epoch::EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
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
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24225566.040); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::DbHz36_41)); 
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
