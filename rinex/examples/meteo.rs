//use rinex::epoch;
use rinex::types::Type;
use rinex::meteo::observable::Observable;

fn main() {
    println!("**************************");
    println!("   (MET) RINEX example    ");
    println!("**************************");

    // example file
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/../test_resources/MET/V2/abvi0010.15m";
    let rinex = rinex::Rinex::from_file(&path).unwrap();

    // header information
    assert_eq!(rinex.header.is_crinex(), false);
    assert_eq!(rinex.header.rinex_type, Type::MeteoData);
    assert_eq!(rinex.header.version.major, 2);
    assert_eq!(rinex.header.version.minor, 11);
    // MeteoData is not tied to a constellation system
    assert_eq!(rinex.header.constellation, None); 
    assert_eq!(rinex.header.leap.is_none(), true); //not provided

    // MeteoData specific information
    let meteo = &rinex.header.meteo
        .as_ref()
        .unwrap();
    let codes = &meteo.codes;
    let sensors = &meteo.sensors;
    println!("####### METEO Sensors #######\n{:#?}", sensors);
    // record analysis
    // ----> determine available observation codes
	println!("\n###### METEO OBS CODES #######\n{:#?}", codes);
    
    // // decimate record: retain data @ 30s interval
    // rinex.resample(std::time::Duration::from_secs(30));
	
    let record = rinex.record.as_meteo().unwrap();

    // list resulting epochs
    let epochs: Vec<_> = record
        .keys()
        .map(|k| k.date)
        .collect();
    println!("\n###### EPOCHS ####\n{:#?}", epochs); 

    // Grab all Temperature measurements from record
    let data : Vec<_> = record
        .iter()
        .map(|(_epoch, obs)| { // record: {key: epochs, values: (array of data) }
            obs.iter() // for all observation
                .find(|(code, _data)| { // key: obscode, value: data 
                    *code == &Observable::Temperature
              })
        })
        .flatten()
        .collect();
    println!("\n##### TEMPERATURE DATA #######\n{:#?}", data);

    // Return (Temperature, Pressure) tuple from record
    let data : Vec<_> = record
        .iter()
        .map(|(_epoch, obs)| { // record: {key: epochs, values: (array of data) }
            obs.iter() // for all observation
                .find(|(code, _)| { // key: obscode, value: data 
                    *code == &Observable::Temperature
              })
        })
        .flatten()
        .collect();
    println!("\n##### (Temp, Pressure) DATA #######\n{:#?}", data);
}
