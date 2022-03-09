use itertools::Itertools;
use rinex::constellation::Constellation;

fn main() {
    println!("**************************");
    println!("   (NAV) RINEX example    ");
    println!("**************************");

    // example file
    let navigation_file = std::path::PathBuf::from(
        env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/navigation.rinex");
    // parse example file
    let rinex = rinex::Rinex::from_file(&navigation_file).unwrap();

    // header information
    assert_eq!(rinex.header.is_crinex(), false);
    assert_eq!(rinex.header.rinex_type, rinex::RinexType::NavigationMessage);
    assert_eq!(rinex.header.version.major, 3);
    assert_eq!(rinex.header.constellation, Constellation::Mixed); 
    // leap second field for instance
    // is a major > 3 optionnal field
    assert_eq!(rinex.header.leap.is_some(), true);

    // (NAV) record analysis
    let record = rinex.record.as_nav()
        .unwrap();

    //////////////////////////////
    // basic record browsing
    //////////////////////////////
                                // record = HashMap<Epoch <()>>
    for entry in record.iter() { // over all epochs
        let epoch = entry.0;
        println!("Found epoch: `{:#?}`", epoch); 
                                        // epoch = HashMap<Sv, ()>
        for vehicule in entry.1.iter() { // over all sat. vehicules
            let sv = vehicule.0;
            println!("Found sat vehicule: `{:#?}`", sv); 
            let data = vehicule.1;
            println!("Data: `{:#?}`", data); 
        }
    }

    //////////////////////////////////////
    // basic hashmap [indexing] 
    // is a quick method to grab some data
    //////////////////////////////////////

    // match a specific `epoch`
    //  * `epoch` is a chrono::NaiveDateTime alias
    //     therefore one can use any chrono::NaiveDateTime method
    let to_match = rinex::epoch::from_string("21 01 01 09 00 00")
        .unwrap();
    //    ---> retrieve all data for desired `epoch`
    //         using direct hashmap[indexing]
    let matched = &record[&to_match];
    println!("\n------------- Matching epoch \"{:?}\" ----------\n{:#?}", to_match, matched); 
    
    // ----> zoom in on `B07` vehicule for that particular `epoch` 
    let to_match = rinex::record::Sv::new(Constellation::Beidou, 0x07);
    let matched = &matched[&to_match];
    println!("\n------------- Adding Sv filter \"{:?}\" to previous epoch filter ----------\n{:#?}", to_match, matched); 
    // ----> zoom in on `B07` clock drift for that `epoch`
    let matched = &matched["ClockDrift"];
    println!("\n------------- \"clockDrift\" data from previous set ----------\n{:#?}", matched); 
    
    ///////////////////////////////////////////////////
    // advanced:
    // iterators + filters allow complex
    // pattern matching, data filtering and extraction
    ///////////////////////////////////////////////////
    let record = rinex.record
        .as_nav()
        .unwrap();

    // list all epochs
    let epochs: Vec<_> = record
        .keys()
        .sorted()
        .collect();
    println!("\n------------- Epochs ----------\n{:#?}", epochs); 
    
    // extract all data for `E04` vehicule 
    let to_match = rinex::record::Sv::new(Constellation::Galileo, 0x04);
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
    // for `E04` vehicule accross entire record
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| {
            sv.iter() // for all sv
                .find(|(&sv, _)| sv == to_match) // match `E04`
                .map(|(_, data)| (&data["ClockBias"],&data["ClockDrift"]))
        })
        .flatten()
        .collect();
    println!("\n------------- \"{:?}\" (bias,drift)----------\n{:#?}", to_match, matched); 
    
    // extract all data tied to `Beidou` constellation
    let to_match = Constellation::Beidou;
    let matched : Vec<_> = record
        .iter()
        .map(|(_epoch, sv)| {
            sv.iter() // for all sv
                .find(|(&sv, _)| sv.constellation == to_match) // match `Rxx`
        })
        .flatten()
        .collect();
    println!("\n------------- Constellation: \"{:?}\" ----------\n{:#?}", to_match, matched); 
}
