#[cfg(test)]
mod test {
    use rinex::prelude::*;
    use rinex::epoch::str2date;
    use std::str::FromStr;
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
		let epoch = Epoch {
			date: str2date("2017 01 01 0 0 0.0").unwrap(),
			flag: EpochFlag::Ok,
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
		let epoch = Epoch {
			date: str2date("2017 01 01 3 33 40.0").unwrap(),
			flag: EpochFlag::Ok,
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
		let epoch = Epoch {
			date: str2date("2017 01 01 6 9 10.0").unwrap(),
			flag: EpochFlag::Ok,
		};
		let epoch = record.get(&epoch);
		assert_eq!(epoch.is_some(), true);
		let (clk_offset, epoch) = epoch.unwrap();
		assert_eq!(clk_offset.is_none(), true);
		assert_eq!(epoch.len(), 11);
    }
	#[test]
	fn v2_npaz3550_21o() {
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
		let epoch = Epoch {
			date: str2date("2021 12 21 0 0 0.0").unwrap(),
			flag: EpochFlag::Ok,
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
	//#[test]
	fn v2_rovn0010_21o() {
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
		let epoch = Epoch {
			date: str2date("2021 01 01 0 0 0.0").unwrap(),
			flag: EpochFlag::Ok,
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
        //C2 
		let observed = observations.get(&String::from("C2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24225562.932); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap()));
        //C5 [missing]
		let observed = observations.get(&String::from("C5"));
		assert_eq!(observed.is_none(), true);
        //L1
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 127306204.852); 
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN)); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap()));
        //L2
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 99199629.819); 
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN)); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("4").unwrap()));
        //L5 [missing]
		let observed = observations.get(&String::from("L5"));
		assert_eq!(observed.is_none(), true);
        //P1
		let observed = observations.get(&String::from("P1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24225565.620); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("4").unwrap()));
        //P2
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 24225563.191); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("4").unwrap()));
        //S1
		let observed = observations.get(&String::from("S1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 40.586); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
        //S2
		let observed = observations.get(&String::from("S2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 25.564); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
        //S5 (missing)
		let observed = observations.get(&String::from("S5"));
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
		let observed = observations.get(&String::from("C1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 23126824.976); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap())); 
		let observed = observations.get(&String::from("C2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 23126830.088); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap())); 
		let observed = observations.get(&String::from("C5"));
		assert_eq!(observed.is_none(), true);
        //L1,L2,L5
		let observed = observations.get(&String::from("L1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 123669526.377); 
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN)); 
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap())); 
		let observed = observations.get(&String::from("L2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 96187435.849); 
		assert_eq!(observed.lli, Some(LliFlags::OK_OR_UNKNOWN));
		assert_eq!(observed.ssi, Some(Ssi::from_str("6").unwrap())); 
		let observed = observations.get(&String::from("L5"));
		assert_eq!(observed.is_none(), true);
        //P1, P2
		let observed = observations.get(&String::from("P1"));
		assert_eq!(observed.is_none(), true);
		let observed = observations.get(&String::from("P2"));
		assert_eq!(observed.is_none(), true);
        //S1,S2,S5
		let observed = observations.get(&String::from("S1"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 41.931); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		let observed = observations.get(&String::from("S2"));
		assert_eq!(observed.is_some(), true);
		let observed = observed.unwrap();
		assert_eq!(observed.obs, 39.856); 
		assert_eq!(observed.lli, None); 
		assert_eq!(observed.ssi, None); 
		let observed = observations.get(&String::from("S5"));
		assert_eq!(observed.is_none(), true);
	}
    #[test]
    fn v3_duth0630() {
        let resource = env!("CARGO_MANIFEST_DIR")
            .to_owned()
            + "/../test_resources/OBS/V3/DUTH0630.22O";
        let rinex = Rinex::from_file(&resource);
        assert_eq!(rinex.is_ok(), true);
        let rinex = rinex
            .unwrap();
        assert_eq!(rinex.header.obs.is_some(), true);
        let obs = rinex.header.obs.as_ref().unwrap();
        let observables = obs.codes.get(&Constellation::Glonass);
        assert_eq!(observables.is_some(), true);
        let observables = observables
            .unwrap();
        let expected: Vec<_> = vec!["C1C","L1C","D1C","S1C","C2P","L2P","D2P","S2P"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(observables, &expected);
        let observables = obs.codes.get(&Constellation::GPS);
        assert_eq!(observables.is_some(), true);
        let observables = observables
            .unwrap();
        let expected: Vec<_> = vec!["C1C","L1C","D1C","S1C","C2W","L2W","D2W","S2W"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(observables, &expected);
        let record = rinex.record.as_obs();
        assert_eq!(record.is_some(), true);
        let record = record
            .unwrap();
        assert_eq!(record.len(), 3);
		
        let epoch = Epoch {
			date: str2date("2022 03 04 0 0 0.0").unwrap(),
			flag: EpochFlag::Ok,
		};
        let e = record.get(&epoch);
        assert_eq!(e.is_some(), true);
        let (clk, vehicules) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicules.len(), 18);
        
        let g01 = Sv { constellation: Constellation::GPS, prn: 01 };
        let g01 = vehicules.get(&g01);
        assert_eq!(g01.is_some(), true);
        let data = g01.unwrap();
        let c1c = data.get("C1C");
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 20243517.560);
        assert_eq!(c1c.lli.is_none(), true);
        assert_eq!(c1c.ssi.is_none(), true);

        let l1c = data.get("L1C");
        assert_eq!(l1c.is_some(), true);
        let l1c = l1c.unwrap();
        assert_eq!(l1c.obs, 106380411.418);
        assert_eq!(l1c.lli, Some(LliFlags::OK_OR_UNKNOWN));
        assert_eq!(l1c.ssi, Some(Ssi::from_str("8").unwrap()));

        let g03 = Sv { constellation: Constellation::GPS, prn: 03 };
        let g03 = vehicules.get(&g03);
        assert_eq!(g03.is_some(), true);
        let data = g03.unwrap();
        let c1c = data.get("C1C");
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 20619020.680);
        assert_eq!(c1c.lli.is_none(), true);
        assert_eq!(c1c.ssi.is_none(), true);
        
        let l1c = data.get("L1C");
        assert_eq!(l1c.is_some(), true);
        let l1c = l1c.unwrap();

        let g04 = Sv { constellation: Constellation::GPS, prn: 04 };
        let g04 = vehicules.get(&g04);
        assert_eq!(g04.is_some(), true);
        let data = g04.unwrap();
        let c1c = data.get("C1C");
        assert_eq!(c1c.is_some(), true);
        let c1c = c1c.unwrap();
        assert_eq!(c1c.obs, 21542633.500);
        assert_eq!(c1c.lli.is_none(), true);
        assert_eq!(c1c.ssi.is_none(), true);
        
        let l1c = data.get("L1C");
        assert_eq!(l1c.is_some(), true);
        let l1c = l1c.unwrap();

        let epoch = Epoch {
			date: str2date("2022 03 04 00 28 30.0").unwrap(),
			flag: EpochFlag::Ok,
		};
        let e = record.get(&epoch);
        assert_eq!(e.is_some(), true);
        let (clk, vehicules) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicules.len(), 17);
		
        let epoch = Epoch {
			date: str2date("2022 03 04 00 57 0.0").unwrap(),
			flag: EpochFlag::Ok,
		};
        let e = record.get(&epoch);
        assert_eq!(e.is_some(), true);
        let (clk, vehicules) = e.unwrap();
        assert_eq!(clk.is_none(), true);
        assert_eq!(vehicules.len(), 17);
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
        let epoch = Epoch {
            date: str2date("2022 06 08 10 00 00.0000000").unwrap(),
            flag: EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);
        
        // EPOCH[2]
        let epoch = Epoch {
            date: str2date("2022 06 08 10 00 30.0000000").unwrap(),
            flag: EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 49);
        
        // EPOCH[3]
        let epoch = Epoch {
            date: str2date("2022 06 08 10 01 00.0000000").unwrap(),
            flag: EpochFlag::Ok,
        };
        let epoch = record.get(&epoch);
        assert_eq!(epoch.is_some(), true);
        let (clk_offset, epoch) = epoch.unwrap();
        assert_eq!(clk_offset.is_none(), true);
        assert_eq!(epoch.len(), 47);
    }
}
