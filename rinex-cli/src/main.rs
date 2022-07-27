//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;
use gnuplot::{Figure}; // Caption};
//use gnuplot::{Color, PointSymbol, LineStyle, DashType};
//use gnuplot::{PointSize, LineWidth}; // AxesCommon};

use rinex::Rinex;
use rinex::sv::Sv;
use rinex::observation;
use rinex::types::Type;
use rinex::epoch;
use rinex::constellation::{Constellation, augmentation::sbas_selection_helper};

pub fn main () -> Result<(), Box<dyn std::error::Error>> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let plot = matches.is_present("plot");
    let pretty = matches.is_present("pretty");

    // files
    let filepaths : Option<Vec<&str>> = match matches.is_present("filepath") {
        true => {
            Some(matches.value_of("filepath")
                .unwrap()
                    .split(",")
                    .collect())
        },
        false => None,
    };

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
    let decimate_ratio = matches.is_present("decim-ratio");
    let decimate_interval = matches.is_present("decim-interval");
    let obscodes_display = matches.is_present("obscodes");

    // teqc ops
    let merge = matches.is_present("merge");
    let splice = matches.is_present("splice");
    let split = matches.is_present("split");
    let _split_epoch : Option<epoch::Epoch> = match matches.value_of("split") {
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
    
    let teqc_ops = merge | split | splice;
    
    ///////////////////////////////////////////////////////////////
    // Filters 
    ///////////////////////////////////////////////////////////////
    // `Epoch`
    let epoch_display = matches.is_present("epoch");
    let epoch_ok_filter = matches.is_present("epoch-ok");
    let epoch_nok_filter = matches.is_present("epoch-nok");
    
    // Constell
    let constell_filter : Option<Vec<Constellation>> = match matches.value_of("constellation") {
        Some(s) => {
            let constellations: Vec<&str> = s.split(",").collect();
            let mut c_filters : Vec<Constellation> = Vec::new();
            for c in constellations {
                if let Ok(constell) = Constellation::from_3_letter_code(c) {
                    c_filters.push(constell)
                } else if let Ok(constell) = Constellation::from_1_letter_code(c) {
                    c_filters.push(constell)
                }
            }
            Some(c_filters)
        },
        _ => None,
    };
    
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
    
    // Other
    let sbas = matches.is_present("sbas"); 

    ////////////////////////////////////////
    // Process arguments that do not require 
    // the parser
    //   - sbas
    ////////////////////////////////////////
    if sbas {
        let sbas = matches.value_of("sbas")
            .unwrap();
        let items :Vec<&str> = sbas.split(",").collect();
        if items.len() != 2 {
            panic!("Command line error: \"--sbas latitude, longitude\" expected");
        }
        let coordinates :Vec<f64> = items
            .iter()
            .map(|s| {
                if let Ok(f) = f64::from_str(s.trim()) {
                    f
                } else {
                    panic!("Command line error: expecting coordinates in decimal degrees")
                }
            })
            .collect();
        let sbas = sbas_selection_helper(coordinates[0], coordinates[1]);
        if let Some(sbas) = sbas {
            println!("SBAS for given coordinates: {:?}", sbas);
        } else {
            println!("No SBAS augmentation for given coordinates");
        }
        return Ok(()); // interrupt before parser section
            // to make --fp optional
    }

    /////////////////////////////////////////
    // from now on: parser is always involved
    //  --fp is a requirement
    /////////////////////////////////////////
    let filepaths = filepaths.unwrap();

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

    ///////////////////////////////////////////////////
    // [1] resampling: reduce data quantity 
    ///////////////////////////////////////////////////
    if decimate_interval {
        let hms = matches.value_of("decim-interval").unwrap();
        let hms : Vec<_> = hms.split(":").collect();
        let (h,m,s) = (
            u64::from_str_radix(hms[0], 10).unwrap(),
            u64::from_str_radix(hms[1], 10).unwrap(),
            u64::from_str_radix(hms[2], 10).unwrap(),
        );
        let interval = std::time::Duration::from_secs(h*3600 + m*60 +s);
        rinex.decimate_by_interval_mut(interval)
    }
    if decimate_ratio {
        let r = u32::from_str_radix(matches.value_of("decim-ratio").unwrap(), 10).unwrap();
        rinex.decimate_by_ratio_mut(r)
    }

    ///////////////////////////////////////////////////////////
    // [2] filtering: reduce data quantity,
    //  focus on data of interest
    //  Doing this prior anything else,
    //  makes merge() or production work on the resulting data
    ///////////////////////////////////////////////////////////
    if epoch_ok_filter {
        rinex
            .epoch_ok_filter_mut()
    }
    if epoch_nok_filter {
        rinex
            .epoch_nok_filter_mut()
    }
    if let Some(ref filter) = constell_filter {
        rinex
            .constellation_filter_mut(filter.to_vec())
    }
    if let Some(ref filter) = sv_filter {
        rinex
            .space_vehicule_filter_mut(filter.to_vec())
    }
    if let Some(ref filter) = obscode_filter {
        rinex
            .observable_filter_mut(filter.to_vec())
    }
    if let Some(lli) = lli {
        let mask = rinex::observation::record::LliFlags::from_bits(lli)
            .unwrap();
        rinex
            .lli_filter_mut(mask)
    }
    if let Some(ssi) = ssi {
        rinex
            .minimum_sig_strength_filter_mut(ssi)
    }
        
    if split {
    /*
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
    */
    }

    if splice {
       println!("splice is WIP"); 
    }
    
    if !teqc_ops {
        // User did not request a `teqc` like / special ops
        // We generate desired output on each --fp entry
        if header {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rinex.header).unwrap())
            } else {
                println!("{}", serde_json::to_string_pretty(&rinex.header).unwrap())
            }
        }
        if epoch_display {
            let epochs = rinex.epochs();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&epochs).unwrap())
            } else {
                println!("{}", serde_json::to_string(&epochs).unwrap())
            }
        }
        if obscodes_display {
            let observables = rinex.observables();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&observables).unwrap())
            } else {
                println!("{}", serde_json::to_string(&observables).unwrap())
            }
        }
        if !epoch_display && !obscodes_display && !header { 
            match rinex.header.rinex_type {
                // display somehow (either graphically or print())
                // remaining data
                // TODO: improve please
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
        if merged.merge_mut(&to_merge[i]).is_err() {
            panic!("Failed to merge {} into {}", filepaths[i], filepaths[0])
        }
    }

    if merge {
        // User requested teqc::merge()
        // we extract desired information off the merged record
        if header {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&merged.header).unwrap())
            } else {
                println!("{}", serde_json::to_string(&merged.header).unwrap())
            }
        }
        if obscodes_display {
            let observables = merged.observables();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&observables).unwrap())
            } else {
                println!("{}", serde_json::to_string(&observables).unwrap())
            }
        }
        if epoch_display {
            let e = merged.epochs();
            if pretty {
                println!("{}", serde_json::to_string_pretty(&e).unwrap())
            } else {
                println!("{}", serde_json::to_string(&e).unwrap())
            }
        }
        if merged.to_file("merged.txt").is_err() {
            panic!("Failed to write MERGED RINEX to \"merged.txt\"")
        }
    }
    Ok(())
}// main
