//! Command line tool to parse and analyze `RINEX` files.    
//! Refer to README for command line arguments.    
//! Based on crate <https://github.com/gwbres/rinex>     
//! Homepage: <https://github.com/gwbres/rinex-cli>
use clap::App;
use clap::load_yaml;
use std::str::FromStr;
use gnuplot::{Figure}; // Caption};
use itertools::Itertools;
//use gnuplot::{Color, PointSymbol, LineStyle, DashType};
//use gnuplot::{PointSize, LineWidth}; // AxesCommon};

use thiserror::Error;
use rinex::Rinex;
use rinex::sv::Sv;
use rinex::observation;
use rinex::types::Type;
use rinex::epoch;
use rinex::constellation::{Constellation};

mod ascii_plot;
use ascii_plot::ascii_plot;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse datetime")]
    ChronoParseError(#[from] chrono::format::ParseError),
    #[error("std::io error")]
    IoError(#[from] std::io::Error),
}

/// Parses an std::time::Duration from user input
fn parse_duration (content: &str) -> Result<std::time::Duration, std::io::Error> {
    let hms : Vec<_> = content.split(":").collect();
    if hms.len() == 3 {
        if let Ok(h) =  u64::from_str_radix(hms[0], 10) {
            if let Ok(m) =  u64::from_str_radix(hms[1], 10) {
                if let Ok(s) =  u64::from_str_radix(hms[2], 10) {
                    return Ok(std::time::Duration::from_secs(h*3600 + m*60 +s))
                }
            }
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to parse %HH:%MM:%SS"))
}

/// Parses an chrono::NaiveDateTime from user input
fn parse_datetime (content: &str) -> Result<chrono::NaiveDateTime, chrono::format::ParseError> {
    chrono::NaiveDateTime::parse_from_str(content, "%Y-%m-%d %H:%M:%S")
}

/// Parses an `epoch` from user input
fn parse_epoch (content: &str) -> Result<epoch::Epoch, Error> {
    let format = "YYYY-MM-DD HH:MM:SS";
    if content.len() > format.len() { // an epoch flag was given
        Ok(epoch::Epoch {
            date: parse_datetime(&content[0..format.len()])?,
            flag: epoch::EpochFlag::from_str(&content[format.len()..].trim())?,
        })
    } else { // no epoch flag given
        // --> we associate an Ok flag
        Ok(epoch::Epoch {
            date: parse_datetime(content)?,
            flag: epoch::EpochFlag::Ok,
        })
    }
}

pub fn main () -> Result<(), Error> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let plot = matches.is_present("plot");
    let pretty = matches.is_present("pretty");

    // files (in)
    let filepaths : Option<Vec<&str>> = match matches.is_present("filepath") {
        true => {
            Some(matches.value_of("filepath")
                .unwrap()
                    .split(",")
                    .collect())
        },
        false => None,
    };
    // files (out)
    let _output : Option<Vec<&str>> = match matches.is_present("output") {
        true => {
            Some(matches.value_of("output")
                .unwrap()
                    .split(",")
                    .collect())
        },
        false => None,
    };

    //let mut fig = Figure::new();

    // RINEX 
    let header = matches.is_present("header");
    let decimate_ratio = matches.is_present("decim-ratio");
    let decimate_interval = matches.is_present("decim-interval");
    let observables_display = matches.is_present("observables");
    let sv_display = matches.is_present("list-sv");
    let sv_per_epoch_display = matches.is_present("list-sv-epoch");
    let clock_offsets = matches.is_present("clock-offsets");
    let gaps = matches.is_present("gaps");
    let largest_gap = matches.is_present("largest-gap");
    let sampling_interval = matches.is_present("sampling-interval");
    let cycle_slips = matches.is_present("cycle-slips");

    // processing ops
    let diff = matches.is_present("diff");
    let ddiff = matches.is_present("ddiff");
    let confirmed_cycle_slips = matches.is_present("confirmed-cycle-slips");

    // teqc ops
    let teqc_plot = matches.is_present("teqc-plot");
    let merge = matches.is_present("merge");
    let splice = matches.is_present("splice");
    let split = matches.is_present("split");
    let split_epoch : Option<epoch::Epoch> = match matches.value_of("split") {
        Some(s) => {
            if let Ok(e) = parse_epoch(s) {
                Some(e)
            } else {
                None
            }
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
    
    /////////////////////////////////////////
    // parse every --fp entry,
    // apply desired filter ops
    /////////////////////////////////////////
    let filepaths = filepaths.unwrap();
    let mut queue : Vec<Rinex> = Vec::new();

for fp in &filepaths {
    let path = std::path::PathBuf::from(fp);
    //fig.set_title(path.file_name().unwrap().to_str().unwrap());
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

    ///////////////////////////////////////////////////
    // [1] resampling: reduce data quantity 
    ///////////////////////////////////////////////////
    if decimate_interval {
        let hms = matches.value_of("decim-interval").unwrap();
        let interval = parse_duration(hms)?;
        //rinex.decimate_by_interval_mut(interval)
    }
    if decimate_ratio {
        let r = u32::from_str_radix(matches.value_of("decim-ratio").unwrap(), 10).unwrap();
        //rinex.decimate_by_ratio_mut(r)
    }

    ///////////////////////////////////////////////////////////
    // [2] filtering: reduce data quantity,
    //  focus on data of interest
    //  Doing this prior anything else,
    //  makes merge() or production work on the resulting data
    ///////////////////////////////////////////////////////////
    if epoch_ok_filter {
        rinex // OK filter
            .epoch_ok_filter_mut()
    }
    if epoch_nok_filter {
        rinex // NOK filter
            .epoch_nok_filter_mut()
    }
    if let Some(ref filter) = constell_filter {
        rinex // apply desired constellation filter
            .constellation_filter_mut(filter.to_vec())
    }
    if let Some(ref filter) = sv_filter {
        rinex // apply desired vehicule filter
            .space_vehicule_filter_mut(filter.to_vec())
    }
    if let Some(ref filter) = obscode_filter {
        rinex // filters out undesired observables
            .observable_filter_mut(filter.to_vec())
    }
    if let Some(lli) = lli {
        let mask = rinex::observation::record::LliFlags::from_bits(lli)
            .unwrap();
        rinex // apply desired LLI filter
            .lli_filter_mut(mask)
    }
    if let Some(ssi) = ssi {
        rinex // apply desired sig strength filter
            .minimum_sig_strength_filter_mut(ssi)
    }

    // push into work queue
    queue.push(rinex);
}// for all files

    /////////////////////////////////////
    // ops that require only 1 file
    /////////////////////////////////////
    for i in 0..queue.len() {
        if split {
            if let Some(epoch) = split_epoch {
                let s = queue[i].split_at_epoch(epoch);
                if let Ok((r0, r1)) = s {
                    if r0.to_file("split1.txt").is_err() {
                        panic!("failed to produce split1.txt")
                    }
                    if r1.to_file("split2.txt").is_err() {
                        panic!("failed to produce split1.txt")
                    }
                } else {
                    panic!("split_at_epoch failed with {:#?}", s);
                }
            }
        }
        
        if splice {

        }
        
        //////////////////////////////////////////
        // ops that might run on a single file
        //////////////////////////////////////////
        let mut at_least_one_op = false;
        if header {
            at_least_one_op = true;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].header).unwrap())
            } else {
                println!("{}", serde_json::to_string_pretty(&queue[i].header).unwrap())
            }
        }
        if epoch_display {
            at_least_one_op = true;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].epochs()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&queue[i].epochs()).unwrap())
            }
        }
        if observables_display {
            at_least_one_op = true;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].observables()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&queue[i].observables()).unwrap())
            }
        }
        if sv_display {
            at_least_one_op = true;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].space_vehicules()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&queue[i].space_vehicules()).unwrap())
            }
        }
        if clock_offsets {
            at_least_one_op = true;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].receiver_clock_offsets()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&queue[i].receiver_clock_offsets()).unwrap())
            }
        }
        if sv_per_epoch_display {
            at_least_one_op = true;
            if pretty {
            //    println!("{}", serde_json::to_string_pretty(&queue[i].space_vehicules_per_epoch()).unwrap())
            } else {
            //    println!("{}", serde_json::to_string(&queue[i].space_vehicules_per_epoch()).unwrap())
            }
        }
        if gaps {
            at_least_one_op = true;
            println!("{:#?}", queue[i].data_gaps());
        }
        if largest_gap {
            at_least_one_op = true;
            println!("{:#?}", queue[i].largest_data_gap_duration());
        }
    
        if teqc_plot {
            at_least_one_op = true;
            println!("{}", ascii_plot(ascii_plot::DEFAULT_X_WIDTH, &queue[i], None));
        }
        
        if cycle_slips {
            at_least_one_op = true;
            println!("{:#?}", queue[i].cycle_slips());
        }

        if !at_least_one_op {
            // print remaining record data
            if pretty {
                println!("{}", serde_json::to_string_pretty(&queue[i].record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&queue[i].record).unwrap())
            }
        }
    } // 1file ops

    /////////////////////////////////////
    // ops that require 2 files
    /////////////////////////////////////
    for i in 0..queue.len()/2 {
        let q_2p = &queue[i*2];
        let q_2p1 = &queue[i*2+1]; 
        if diff {
            if let Ok(q) = q_2p.double_diff(q_2p1) {
                // print remaining record data
                if pretty {
                    println!("{}", serde_json::to_string_pretty(&q.record).unwrap())
                } else {
                    println!("{}", serde_json::to_string(&q.record).unwrap())
                }
            } 
        }
        if ddiff {
            if let Ok(q) = q_2p.double_diff(q_2p1) {
                // print remaining record data
                if pretty {
                    println!("{}", serde_json::to_string_pretty(&q.record).unwrap())
                } else {
                    println!("{}", serde_json::to_string(&q.record).unwrap())
                }
            }
        } 
        if confirmed_cycle_slips {
            if let Ok(slips) = q_2p.confirmed_cycle_slips(q_2p1) {
                if pretty {
                    println!("{}", serde_json::to_string_pretty(&slips).unwrap())
                } else {
                    println!("{}", serde_json::to_string(&slips).unwrap())
                }
            }
        }
        /*if merge {
            if q0.merge(q1).is_err() {
                panic!("Failed to merge {} into {}", filepaths[i*2], filepaths[i*2+1]);
            }
        }*/
    }//2file ops
    
    Ok(())
}// main

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_duration_parser() {
        let duration = parse_duration("00:30:00");
        assert_eq!(duration.is_ok(), true);
        let duration = duration.unwrap();
        assert_eq!(duration, std::time::Duration::from_secs(30*60));
        let duration = parse_duration("30:00");
        assert_eq!(duration.is_err(), true);
        let duration = parse_duration("00 30 00");
        assert_eq!(duration.is_err(), true);
    }
    #[test]
    fn test_epoch_parser() {
        let epoch = parse_epoch("2022-03-01 00:30:00");
        assert_eq!(epoch.is_ok(), true);
        let epoch = epoch.unwrap();
        assert_eq!(epoch, epoch::Epoch {
            date: parse_datetime("2022-03-01 00:30:00").unwrap(),
            flag: epoch::EpochFlag::Ok,
        });
    }
}
