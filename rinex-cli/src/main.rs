//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;
use std::collections::HashMap;
use gnuplot::{Figure}; // Caption};
//use gnuplot::{Color, PointSymbol, LineStyle, DashType};
//use gnuplot::{PointSize, LineWidth}; // AxesCommon};

use rinex::Rinex;
use rinex::sv::Sv;
use rinex::meteo;
use rinex::navigation;
use rinex::observation;
use rinex::types::Type;
use rinex::epoch;
use rinex::record::Record;
use rinex::constellation::Constellation;

pub fn main () -> Result<(), Box<dyn std::error::Error>> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let plot = matches.is_present("plot");
    let pretty = matches.is_present("pretty");
    let filepaths : Vec<&str> = matches.value_of("filepath")
        .unwrap()
            .split(",")
            .collect();

    let _output : Option<Vec<&str>> = match matches.is_present("output") {
        true => {
            Some(matches.value_of("output")
                .unwrap()
                    .split(",")
                    .collect())
        },
        false => None,
    };

    let mut fig = Figure::new();

    // RINEX 
    let header = matches.is_present("header");
    let resampling = matches.is_present("resampling");
    let decimate = matches.is_present("decimate");

    // SPEC ops
    let merge = matches.is_present("merge");

    let split = matches.is_present("split");
    let split_epoch : Option<epoch::Epoch> = match matches.value_of("split") {
        Some(date) => {
            let offset = 4 +2+1 +2+1 +2+1 +2+1 +2+1; // YYYY-mm-dd-HH:MM:SS 
            let datetime = date[0..offset].to_string();
            let flag : Option<epoch::EpochFlag> = match date.len() > offset {
                true => Some(epoch::EpochFlag::from_str(&date[offset+1..])
                    .unwrap_or(epoch::EpochFlag::Ok)),
                false => None,
            };
            Some(epoch::Epoch {
                date : chrono::NaiveDateTime::parse_from_str(&datetime, "%Y-%m-%d %H:%M:%S")
                    .unwrap(), 
                flag : flag.unwrap_or(epoch::EpochFlag::Ok),
            })
        },
        None => None,
    };
    
    let splice = matches.is_present("splice");

    let spec_ops = merge | split | splice;
    
    // `Epoch`
    let epoch_display = matches.is_present("epoch");
    let epoch_ok_filter = matches.is_present("epoch-ok");
    let epoch_nok_filter = matches.is_present("epoch-nok");
    
    // `Sv`
    let sv_filter : Option<Vec<Sv>> = match matches.value_of("sv") {
        Some(s) => {
            let sv: Vec<&str> = s.split(",").collect();
            let mut sv_filters : Vec<Sv> = Vec::new();
            for s in sv {
                let constell = Constellation::from_str(&s[0..1])
                    .unwrap();
                let prn = u8::from_str_radix(&s[1..], 10)
                    .unwrap();
                sv_filters.push(Sv::new(constell,prn))
            }
            Some(sv_filters)
        },
        _ => None,
    };

    // Data Filters 
    let obscodes_display = matches.is_present("obscodes");
    let obscode_filter : Option<Vec<&str>> = match matches.value_of("codes") {
        Some(s) => Some(s.split(",").collect()),
        _ => None,
    };
    let lli : Option<u8> = match matches.value_of("lli") {
        Some(s) => Some(u8::from_str_radix(s,10).unwrap()),
        _ => None,
    };
    let ssi : Option<observation::record::Ssi> = match matches.value_of("ssi") {
        Some(s) => Some(observation::record::Ssi::from_str(s).unwrap()),
        _ => None,
    };

    let mut index : usize = 0;
    let mut merged: Rinex = Rinex::default();
    let mut to_merge : Vec<Rinex> = Vec::new(); 

for fp in &filepaths {
    let path = std::path::PathBuf::from(fp);
    fig.set_title(path.file_name().unwrap().to_str().unwrap());
    let mut rinex = match path.exists() {
        true => {
            if let Ok(r) = Rinex::from_file(fp) {
                r
            } else {
                println!("Failed to parse file \"{}\"", fp); 
                continue
            }
        },
        false => {
            println!("File \"{}\" does not exist", fp);
            continue
        },
    };

    if index == 0 {
        merged = rinex.clone()
    } else {
        to_merge.push(rinex.clone())
    }
    index += 1;

    // [1] resampling
    if resampling {
       println!("resampling is WIP"); 
        let hms = matches.value_of("resampling").unwrap();
        let hms : Vec<_> = hms.split(":").collect();
        let (h,m,s) = (
            u64::from_str_radix(hms[0], 10).unwrap(),
            u64::from_str_radix(hms[1], 10).unwrap(),
            u64::from_str_radix(hms[2], 10).unwrap(),
        );
        let interval = std::time::Duration::from_secs(h*3600 + m*60 +s);
        rinex.resample(interval)
    }
    if decimate {
       println!("resampling is WIP"); 
        let r = u32::from_str_radix(matches.value_of("decimate").unwrap(), 10).unwrap();
        rinex.decimate(r)
    }

    // [2] epoch::ok filter
    if epoch_ok_filter {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let filtered : Vec<_> = rinex.record
                    .as_obs()
                    .unwrap()
                    .iter()
                    .filter(|(epoch, (_, _))| {
                        epoch.flag.is_ok()
                    })
                    .collect();
                let mut rework = observation::record::Record::new();
                for (e, data) in filtered {
                    rework.insert(*e, data.clone());
                }
                rinex.record = Record::ObsRecord(rework)
            },
            Type::MeteoData => {
                let filtered : Vec<_> = rinex.record
                    .as_meteo()
                    .unwrap()
                    .iter()
                    .filter(|(epoch, _)| {
                        epoch.flag.is_ok()
                    })
                    .collect();
                let mut rework = meteo::record::Record::new();
                for (e, data) in filtered {
                    rework.insert(*e, data.clone());
                }
                rinex.record = Record::MeteoRecord(rework)
            },
            _ => {},
        }
    }
    // [2*] !epoch::ok filter
    if epoch_nok_filter {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let filtered : Vec<_> = rinex.record
                    .as_obs()
                    .unwrap()
                    .iter()
                    .filter(|(epoch, (_, _))| {
                        !epoch.flag.is_ok()
                    })
                    .collect();
                let mut rework = observation::record::Record::new();
                for (e, data) in filtered {
                    rework.insert(*e, data.clone());
                }
                rinex.record = Record::ObsRecord(rework)
            },
            Type::MeteoData => {
                let filtered : Vec<_> = rinex.record
                    .as_meteo()
                    .unwrap()
                    .iter()
                    .filter(|(epoch, _)| {
                        !epoch.flag.is_ok()
                    })
                    .collect();
                let mut rework = meteo::record::Record::new();
                for (e, data) in filtered {
                    rework.insert(*e, data.clone());
                }
                rinex.record = Record::MeteoRecord(rework)
            },
            _ => {},
        }
    }

    // [3] sv filter
    if let Some(ref filter) = sv_filter {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let mut rework = observation::record::Record::new();
                for (epoch, (ck,data)) in rinex.record.as_obs().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, observation::record::ObservationData>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        if filter.contains(sv) {
                            map.insert(*sv, data.clone());
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, (*ck, map));
                    }
                }
                rinex.record = Record::ObsRecord(rework)
            },
            Type::NavigationData => {
                let mut rework = navigation::record::Record::new();
                for (epoch, data) in rinex.record.as_nav().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, navigation::record::ComplexEnum>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        if filter.contains(sv) {
                            map.insert(*sv, data.clone());
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, map);
                    }
                }
                rinex.record = Record::NavRecord(rework)
            },
            _ => {},
        }
    }

    // [4] OBS code filter
    if let Some(ref filter) = obscode_filter {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let mut rework = observation::record::Record::new();
                for (epoch, (ck,data)) in rinex.record.as_obs().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, observation::record::ObservationData>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        let mut inner : HashMap<String, observation::record::ObservationData> = HashMap::new();
                        for (code, data) in data.iter() {
                            if filter.contains(&code.as_str()) {
                                inner.insert(code.clone(), data.clone());
                            }
                        }
                        if inner.len() > 0 {
                            map.insert(*sv, inner);
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, (*ck, map));
                    }
                }
                rinex.record = Record::ObsRecord(rework)
            },
            Type::MeteoData => {
                let mut rework = meteo::record::Record::new();
                for (epoch, data) in rinex.record.as_meteo().unwrap().iter() {
                    let mut map : HashMap<meteo::observable::Observable, f32> = HashMap::new(); 
                    for (code, data) in data.iter() {
                        if filter.contains(&code.to_string().as_str()) {
                            map.insert(code.clone(), *data);
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, map);
                    }
                }
                rinex.record = Record::MeteoRecord(rework)
            },
            Type::NavigationData => {
                let mut rework = navigation::record::Record::new();
                for (epoch, data) in rinex.record.as_nav().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, navigation::record::ComplexEnum>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        let mut inner : HashMap<String, navigation::record::ComplexEnum> = HashMap::new();
                        for (code, data) in data.iter() {
                            if filter.contains(&code.as_str()) {
                                inner.insert(code.clone(), data.clone());
                            }
                        }
                        if inner.len() > 0 {
                            map.insert(*sv, inner);
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, map);
                    }
                }
                rinex.record = Record::NavRecord(rework)
            },
            _ => todo!("Rinex type not fully supported yet"),
        }
    }
    //[4*] LLI filter
    if let Some(lli) = lli {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let mut rework = observation::record::Record::new();
                for (epoch, (ck,data)) in rinex.record.as_obs().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, observation::record::ObservationData>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        let mut inner : HashMap<String, observation::record::ObservationData> = HashMap::new();
                        for (code, data) in data.iter() {
                            if let Some(lli_flags) = data.lli {
                                if lli_flags == lli { 
                                    inner.insert(code.clone(), data.clone());
                                }
                            }
                        }
                        if inner.len() > 0 {
                            map.insert(*sv, inner);
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, (*ck, map));
                    }
                }
                rinex.record = Record::ObsRecord(rework)
            },
            _ => {},
        }
    }
    //[4*] SSI filter
    if let Some(ssi) = ssi {
        match &rinex.header.rinex_type {
            Type::ObservationData => {
                let mut rework = observation::record::Record::new();
                for (epoch, (ck,data)) in rinex.record.as_obs().unwrap().iter() {
                    let mut map : HashMap<Sv, HashMap<String, observation::record::ObservationData>> = HashMap::new();
                    for (sv, data) in data.iter() {
                        let mut inner : HashMap<String, observation::record::ObservationData> = HashMap::new();
                        for (code, data) in data.iter() {
                            if let Some(ssi_value) = data.ssi {
                                if ssi_value >= ssi { 
                                    inner.insert(code.clone(), data.clone());
                                }
                            }
                        }
                        if inner.len() > 0 {
                            map.insert(*sv, inner);
                        }
                    }
                    if map.len() > 0 {
                        rework.insert(*epoch, (*ck, map));
                    }
                }
                rinex.record = Record::ObsRecord(rework)
            },
            _ => {},
        }
    }
        
    if split {
        match rinex.split(split_epoch) {
            Ok(files) => {
                println!("{}", files.len());
                for f in files {
                    if f.to_file("split.txt").is_err() {
                        println!("Failed to write Split record to \"split.txt\"")
                    }
                }
            },
            Err(e) => println!("Split() ops failed with {:?}", e),
        }
    }

    if splice {
       println!("splice is WIP"); 
    }
    
    if !spec_ops {
        if header {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rinex.header).unwrap())
            } else {
                println!("{}", serde_json::to_string_pretty(&rinex.header).unwrap())
            }
        }
        if epoch_display {
            let e = rinex.epochs_iter();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&e).unwrap())
            } else {
                println!("{}", serde_json::to_string(&e).unwrap())
            }
        }
        if obscodes_display {
            match rinex.header.rinex_type {
                Type::ObservationData => {
                    let obscodes = &rinex.header
                        .obs
                        .as_ref()
                            .unwrap()
                            .codes;
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(&obscodes).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(&obscodes).unwrap())
                    }
                },
                Type::MeteoData => {
                    let obscodes = &rinex.header
                        .meteo
                        .as_ref()
                            .unwrap()
                            .codes;
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(&obscodes).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(&obscodes).unwrap())
                    }
                },
                Type::NavigationData => { // (NAV) special procedure: obscodes are not given by header fields
                    let r = rinex.record.as_nav().unwrap();
                    let mut map : HashMap<String, Vec<String>> = HashMap::new();
                    for (_, sv) in r.iter() {
                        let _codes : Vec<String> = Vec::new();
                        for (sv, data) in sv {
                            let codes : Vec<String> = data
                                .keys()
                                .map(|k| k.to_string())
                                .collect();
                            map.insert(
                                sv.constellation.to_3_letter_code().to_string(), 
                                codes);
                        }
                    }
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(&map).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(&map).unwrap())
                    }
                },
                _ => todo!("RINEX type not fully supported yet"),
            };
        }
        if !epoch_display && !obscodes_display && !header { 
            match rinex.header.rinex_type {
                Type::ObservationData => {
                    let r = rinex.record.as_obs().unwrap();
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(r).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(r).unwrap())
                    }
                    if plot {
                        ///////////////////////////////////
                        // determine remaining OBS codes
                        ///////////////////////////////////
                        let mut codes : Vec<String> = Vec::new();
                        r.iter()
                            .for_each(|(_, (_,data))| {
                                data.iter()
                                    .for_each(|(_, data)| {
                                        data.iter()
                                            .for_each(|(code, _)| {
                                                if !codes.contains(code) {
                                                    codes.push(code.to_string())
                                                }
                                            })
                                    })
                                });
                        for code in codes { // for all remaining OBS/physics
                            let _data: Vec<Vec<Vec<f32>>> = Vec::new();// PerSV,PerEpoch,Data
                            r.iter()
                                .for_each(|(_e, (_, sv))| {
                                    sv.iter()
                                        .for_each(|(_sv, _data)| {
                                            
                                        })
                                });
                            fig.save_to_png(&format!("{}.png", code),640,480).unwrap()
                        } // for all remaining OBS/physics
                        /*
                        // for all constellations 
                        for constellation in constellations {
                            // all codes available for this constellation
                            let codes = &obscodes[&constellation]; 
                            // [1] Determine available (remaining) 
                            // types of OBS for this constellation
                            let mut obs_types : Vec<String> = Vec::new();
                            r.iter()
                                .for_each(|(e, (ck, sv))| {
                                    sv.iter()
                                        .for_each(|(sv, data)| {
                                        if sv.constellation == constellation {
                                            data.iter()
                                                .for_each(|(code, _)| {
                                                if !obs_types.contains(code) {
                                                    obs_types.push(code.to_string())
                                                }
                                            })
                                        }
                                    })
                                });
                            for obs_type in obs_types { // for all available OBS
                                // grab OBS raw data
                                let data : HashMap<Sv, f32> = HashMap::new();
                                 r
                                    .iter()
                                    .map(|(_, (_, sv))| {
                                        sv.iter()
                                            .find(|(k,_)| k.constellation == constellation)
                                            .map(|(sv, data)| {
                                                data.iter()
                                                    .find(|(code, _)| *code == &obs_type)
                                                    .map(|(_, data)| {
                                                        data.obs
                                                    })
                                            })
                                            .flatten()
                                    })
                                    .flatten()
                                    .collect();
                                let y : Vec<f32> = (0..z.len()).map(|x| x as f32).collect();
                                let mut axes = fig.axes3d()
                                    .set_x_grid(true)
                                    .set_y_grid(true)
                                    .set_z_grid(true);
                                axes
                                    .lines(x, y.clone(), y.clone(), &[
                                        Caption(&obs_type),
                                        Color("blue"),
                                        PointSize(10.0),
                                        PointSymbol('x'),
                                    ]);
                            }
                        }*/
                    }
                },
                Type::NavigationData => {
                    let r = rinex.record.as_nav().unwrap();
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(r).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(r).unwrap())
                    }
                },
                Type::MeteoData => {
                    let r = rinex.record.as_meteo().unwrap();
                    if pretty {
                        println!("{}", serde_json::to_string_pretty(r).unwrap())
                    } else {
                        println!("{}", serde_json::to_string(r).unwrap())
                    }
                },
                _ => todo!("RINEX type not fully suppported yet"),
            }
        }
    }
}// for all files
    
    // Merge() opt
    for i in 0..to_merge.len() {
        if merged.merge(&to_merge[i]).is_err() {
            println!("Failed to merge {} into {}", filepaths[i], filepaths[0])
        }
    }

    if merge {
        if header {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&merged.header).unwrap())
            } else {
                println!("{}", serde_json::to_string(&merged.header).unwrap())
            }
        }
        if obscodes_display {
            let obs = &merged.header.obs
                .as_ref()
                .unwrap()
                .codes;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&obs).unwrap())
            } else {
                println!("{}", serde_json::to_string(&obs).unwrap())
            }
        }
        if epoch_display {
            let e = merged.epochs_iter();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&e).unwrap())
            } else {
                println!("{}", serde_json::to_string(&e).unwrap())
            }
        }
        if merge {
            if merged.to_file("merged.txt").is_err() {
                println!("Failed to write MERGED RINEX to \"merged.txt\"")
            }
        }
    }
    Ok(())
}// main
