use rinex::epoch;
use rinex::types::Type;
use rinex::constellation::Constellation;

fn main() {
    println!("**************************");
    println!("   (OBS) RINEX example    ");
    println!("**************************");

    // example file
    let fp = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/data/OBS/V3/LARM0630.22O");
    // parse example file
    let rinex = rinex::Rinex::from_file(&fp).unwrap();

    // header information
    assert_eq!(rinex.header.is_crinex(), false);
    assert_eq!(rinex.header.rinex_type, Type::ObservationData);
    assert_eq!(rinex.header.version.major, 3);
    assert_eq!(rinex.header.constellation, Some(Constellation::Mixed)); 
    // leap second field for instance
    // is a major > 3 optionnal field
    assert_eq!(rinex.header.leap.is_some(), true);

    // (OBS) record analysis
    if let Some(ref record) = rinex.record {
        let record = record
            .as_obs()
            .unwrap();
	
	//////////////////////////////
    // basic record browsing
    //////////////////////////////
    for entry in record.iter() { // over all epochs
        let epoch = entry.0;
        println!("Found epoch: `{:#?}`", epoch); 
        // epochs are 2D (1 per epoch)
        //   clock offsets (if any) : Some(f32)
        //   HashMap<Sv, HashMap<String, ObservationData>> 
        //   : list of observation data, indexed by Observation Code
        //     and sorted by Satellite Vehicule
        /*for vehicule in entry.1.iter() { // over all sat. vehicules
            let sv = vehicule.0;
            println!("Found sat vehicule: `{:#?}`", sv); 
            let data = vehicule.1;
            println!("Data: `{:#?}`", data); 
        }*/
    }
    //////////////////////////////////////
    // basic hashmap [indexing] 
    // is a quick method to grab some data
    //////////////////////////////////////

    // match a specific `epoch`
    //  * `epoch` is a chrono::NaiveDateTime alias
    //     therefore one can use any chrono::NaiveDateTime method
    let to_match = epoch::Epoch::new(
        epoch::str2date("22 03 04 00 00 00").unwrap(),
        epoch::EpochFlag::Ok);
    //    ---> retrieve all data for desired `epoch`
    //         using direct hashmap[indexing]
    let matched = &record[&to_match];
    println!("\n------------- Matching epoch \"{:?}\" ----------\n{:#?}", to_match, matched); 

	// ----> determine available observation codes
	//       for Glonass system
	let obs_codes = &rinex.header.obs_codes
		.unwrap()
		[&Constellation::Glonass];
	println!("\n----------- OBS codes for {} system-------\n{:#?}", Constellation::Glonass, obs_codes);
    
    // ----> zoom in on `R24` vehicule for that particular `epoch` 
    let to_match = rinex::record::Sv::new(Constellation::Glonass, 24);
    //let matched = &matched[&to_match];
    println!("\n------------- Adding Sv filter \"{:?}\" to previous epoch filter ----------\n{:#?}", to_match, matched); 
    // ----> grab `R24` "C1C" phase observation for that  `epoch`
    //let matched = &matched["C1C"];
    println!("\n------------- \"C1C\" data from previous set ----------\n{:#?}", matched); 
    
/*
    ///////////////////////////////////////////////////
    // advanced:
    // iterators + filters allow complex
    // pattern matching, data filtering and extraction
    ///////////////////////////////////////////////////
    let record = rinex.record.unwrap();
    let record = record
        .as_nav()
        .unwrap();

    // list all epochs
    let epochs: Vec<_> = record
        .keys()
        .map(|k| k.date)
        .sorted()
        .collect();
    println!("\n------------- Epochs ----------\n{:#?}", epochs); 
    
    // extract all data for `R24` vehicule 
    let to_match = rinex::record::Sv::new(Constellation::Glonass, 24);
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| { // dont care about epoch, sv filter
            sv.iter() // for all sv
                .find(|(&sv, _)| sv == to_match) // match `E04`
        })
        .flatten() // dump non matching data
        .collect();
    println!("\n------------- \"{:?}\" data ----------\n{:#?}", to_match, matched); 
    
    // extract `clockbias` & `clockdrift` fields
    // for `R24` vehicule accross entire record
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| {
            sv.iter() // for all sv
                .find(|(&sv, _)| sv == to_match) // match `R24`
                .map(|(_, data)| ( // create a tuple
                    data["ClockBias"]
                        .as_f32()
                        .unwrap(),
                    data["ClockDrift"]
                        .as_f32()
                        .unwrap(),
                ))
        })
        .flatten()
        .collect();
    println!("\n------------- \"{:?}\" (bias,drift)----------\n{:#?}", to_match, matched); 
    // extract all data tied to `Galileo` constellation
    let to_match = Constellation::Galileo;
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| {
            sv.iter() // for all sv
                .find(|(&sv, _)| sv.constellation == to_match) // match `Rxx`
        })
        .flatten()
        .collect();
    println!("\n------------- Constellation: \"{:?}\" ----------\n{:#?}", to_match, matched); 
    }*/
    }

}
