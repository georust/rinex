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

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse datetime")]
    ChronoParseError(#[from] chrono::format::ParseError),
    #[error("std::io error")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
enum ParseDurationError {
    #[error("format should be %HH:%MM:%SS")]
    InvalidFormat,
    #[error("time internal overflow!")]
    TimeOutOfRange(#[from] time::OutOfRangeError),
}

/// Parses an std::time::Duration from user input
fn parse_duration (content: &str) -> Result<chrono::Duration, ParseDurationError> {
    let hms : Vec<_> = content.split(":").collect();
    if hms.len() == 3 {
        if let Ok(h) =  u64::from_str_radix(hms[0], 10) {
            if let Ok(m) =  u64::from_str_radix(hms[1], 10) {
                if let Ok(s) =  u64::from_str_radix(hms[2], 10) {
                    let std = std::time::Duration::from_secs(h*3600 + m*60 +s);
                    return Ok(chrono::Duration::from_std(std)?)
                }
            }
        }
    }
    Err(ParseDurationError::InvalidFormat)
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

/// Resample given file as possibly requested
fn resample_single_file (rnx: &mut rinex::Rinex, matches: clap::ArgMatches) {
    if let Some(interval) = matches.value_of("decim-interval") {
        let hms = matches.value_of("decim-interval").unwrap();
        if let Ok(interval) = parse_duration(hms) {
            rnx.decimate_by_interval_mut(interval)
        }
    }
    if let Some(r) = matches.value_of("decim-ratio") {
        if let Ok(r) = u32::from_str_radix(r, 10) {
            rnx.decimate_by_ratio_mut(r)
        }
    }
}

/// Apply desired filters
fn apply_filters (rinex: &mut rinex::Rinex, matches: clap::ArgMatches) {
    let epoch_ok_filter = matches.is_present("epoch-ok-filter");
    let epoch_nok_filter = matches.is_present("epoch-nok-filter");
    
    let constell_filter : Option<Vec<Constellation>> = match matches.value_of("constellation-filter") {
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
    
    let sv_filter : Option<Vec<Sv>> = match matches.value_of("sv-filter") {
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

    let observ_filter : Option<Vec<&str>> = match matches.value_of("observ-filter") {
        Some(s) => Some(s.split(",").collect()),
        _ => None,
    };
    let lli_mask : Option<u8> = match matches.value_of("lli-mask") {
        Some(s) => Some(u8::from_str_radix(s,10).unwrap()),
        _ => None,
    };
    let ssi_filter : Option<observation::record::Ssi> = match matches.value_of("ssi-filter") {
        Some(s) => Some(observation::record::Ssi::from_str(s).unwrap()),
        _ => None,
    };
    
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
    if let Some(ref filter) = observ_filter {
        rinex
            .observable_filter_mut(filter.to_vec())
    }
    if let Some(lli) = lli_mask {
        let mask = rinex::observation::record::LliFlags::from_bits(lli)
            .unwrap();
        rinex
            .lli_filter_mut(mask)
    }
    if let Some(ssi) = ssi_filter {
        rinex
            .minimum_sig_strength_filter_mut(ssi)
    }
}

/// Execute user requests on a single file
fn run_single_file_op (rnx: &rinex::Rinex, matches: clap::ArgMatches, print_allowed: bool) {
    let pretty = matches.is_present("pretty");
    let header = matches.is_present("header");
    let decimate_ratio = matches.is_present("decim-ratio");
    let decimate_interval = matches.is_present("decim-interval");
    let observables = matches.is_present("observ");
    let epoch = matches.is_present("epoch");
    let sv = matches.is_present("sv");
    let ssi_range = matches.is_present("ssi-range");
    let constellations = matches.is_present("constellations");
    let sv_per_epoch = matches.is_present("sv-per-epoch");
    let clock_offsets = matches.is_present("clock-offsets");
    let gaps = matches.is_present("gaps");
    let largest_gap = matches.is_present("largest-gap");
    let sampling_interval = matches.is_present("sampling-interval");
    let cycle_slips = matches.is_present("cycle-slips");

    let mut at_least_one_op = false;

    if header {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.header).unwrap())
            } else {
                println!("{}", serde_json::to_string_pretty(&rnx.header).unwrap())
            }
        }
    }
    if epoch {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.epochs()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.epochs()).unwrap())
            }
        }
    }
    if observables {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.observables()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.observables()).unwrap())
            }
        }
    }
    if constellations {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.constellations()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.constellations()).unwrap())
            }
        }
    }
    if sv {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.space_vehicules()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.space_vehicules()).unwrap())
            }
        }
    }
    if ssi_range {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.sig_strength_range()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.sig_strength_range()).unwrap())
            }
        }
    }
    if clock_offsets {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.receiver_clock_offsets()).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.receiver_clock_offsets()).unwrap())
            }
        }
    }
    if sv_per_epoch {
        at_least_one_op = true;
        if print_allowed {
            if pretty {
            //    println!("{}", serde_json::to_string_pretty(&rnx.space_vehicules_per_epoch()).unwrap())
            } else {
            //    println!("{}", serde_json::to_string(&rnx.space_vehicules_per_epoch()).unwrap())
            }
        }
    }
    if gaps {
        at_least_one_op = true;
        if print_allowed {
            println!("{:#?}", rnx.data_gaps());
        }
    }
    if largest_gap {
        at_least_one_op = true;
        if print_allowed {
            println!("{:#?}", rnx.largest_data_gap_duration());
        }
    }

    if cycle_slips {
        at_least_one_op = true;
        if print_allowed {
            println!("{:#?}", rnx.cycle_slips());
        }
    }

    if !at_least_one_op {
        // print remaining record data
        if print_allowed {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        }
    }
}

/// Execute `teqc` ops on a single file
fn run_single_file_teqc_op (rnx: &rinex::Rinex, matches: clap::ArgMatches) {
    let ascii_plot = matches.is_present("ascii-plot");
    let merge = matches.is_present("merge");
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
    if ascii_plot {
        println!("{}", ascii_plot::ascii_plot(ascii_plot::DEFAULT_X_WIDTH, &rnx, None));
    }
}

/// Execute user requests on two files
fn run_double_file_op (rnx_a: &rinex::Rinex, rnx_b: &rinex::Rinex, matches: clap::ArgMatches) {
    let pretty = matches.is_present("pretty");
    let diff = matches.is_present("diff");
    let ddiff = matches.is_present("ddiff");
    let confirm_cycle_slips = matches.is_present("confirm-cycle-slips");
    if diff {
        if let Ok(rnx) = rnx_a.diff(rnx_b) {
            // print remaining record data
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        } 
    }
    if ddiff {
        if let Ok(rnx) = rnx_a.double_diff(rnx_b) {
            // print remaining record data
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rnx.record).unwrap())
            } else {
                println!("{}", serde_json::to_string(&rnx.record).unwrap())
            }
        }
    } 
    if confirm_cycle_slips {
        /*if let Ok(slips) = rnx_a.confirmed_cycle_slips(rnx_b) {
            if pretty {
                println!("{}", serde_json::to_string_pretty(&slips).unwrap())
            } else {
                println!("{}", serde_json::to_string(&slips).unwrap())
            }
        }*/
    }
    /*if merge {
        if q0.merge(q1).is_err() {
            panic!("Failed to merge {} into {}", filepaths[i*2], filepaths[i*2+1]);
        }
    }*/
}

pub fn main () -> Result<(), Error> {
	let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml);
	let matches = app.get_matches();

    // General 
    let plot = matches.is_present("plot");

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

    //TODO graphical view
    //let mut fig = Figure::new();

    let filepaths = filepaths.unwrap();
    let mut queue : Vec<Rinex> = Vec::new();

    ////////////////////////////////////////
    // Parse, filter, resample
    ////////////////////////////////////////
    let mut index = 0;
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
        resample_single_file(&mut rinex, matches.clone());
        apply_filters(&mut rinex, matches.clone());
        queue.push(rinex);
        index += 1
    }

    /////////////////////////////////////
    // ops that require only 1 file
    /////////////////////////////////////
    for i in 0..queue.len() {
        let mut print_allowed = true;
        print_allowed &= !matches.is_present("merge");
        print_allowed &= !matches.is_present("diff");
        print_allowed &= !matches.is_present("ddiff");
        print_allowed &= !matches.is_present("ascii-plot");
        run_single_file_op(&queue[i], matches.clone(), print_allowed);
        run_single_file_teqc_op(&queue[i], matches.clone());
    }

    /////////////////////////////////////
    // ops that require 2 files
    /////////////////////////////////////
    for i in 0..queue.len()/2 {
        let q_2p = &queue[i*2];
        let q_2p1 = &queue[i*2+1]; 
        run_double_file_op(&q_2p, &q_2p1, matches.clone());
    }
    
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
        assert_eq!(duration, chrono::Duration::minutes(30));
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
