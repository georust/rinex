use rinex::sv;
use rinex::epoch;
use rinex::types::Type;
use rinex::constellation::Constellation;

fn main() {
    println!("**************************");
    println!("   (MET) RINEX example    ");
    println!("**************************");

    // example file
    let path = env!("CARGO_MANIFEST_DIR").to_owned() + "/data/MET/V2/abvi0010.15m";
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
    let sensors = &rinex.header.sensors;
    println!("####### METEO Sensors #######\n{:#?}", sensors);

    // record analysis
    let record = rinex.decimate(std::time::Duration::from_secs(30));
	// ----> determine available observation codes
	let obs_codes = &rinex.header.met_codes.unwrap();
	println!("\n###### METEO OBS CODES #######\n{:#?}", obs_codes);
    
    // decimate record: retain data @ 30s interval
    let record = record.as_meteo().unwrap();

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
                .filter(|(code, _data)| { // key: obscode, value: data 
                    rinex::is_temperature_obs_code!(code.as_str())
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
                .flat_map(|(code, data)| { // key: obscode, value: data 
                    if rinex::is_temperature_obs_code!(code) {
                        Some(("temp",data))
                    } else if rinex::is_pressure_obs_code!(code) {
                        Some(("press",data))
                    } else {
                        None
                    }
              })
        })
        .flatten()
        .collect();
    println!("\n##### (Temp, Pressure) DATA #######\n{:#?}", data);
}
