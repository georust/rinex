use std::str::FromStr;
use chrono::{Duration, NaiveDateTime};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("format should be %HH:%MM:%SS to describe a duration")]
    InvalidDurationFormat,
}

fn parse_duration(args: &str) -> Result<Duration, Error> {
    let hms:Vec<&str> = args
        .split(":")
        .collect();
    if hms.len() != 3 {
        return Err(Error::InvalidDurationFormat)
    }
    
    if let Ok(h) = u64::from_str_radix(hms[0], 10) {
        if let Ok(m) = u64::from_str_radix(hms[1], 10) {
            if let Ok(s) = u64::from_str_radix(hms[2], 10) {
                let std = std::time::Duration::from_secs(
                    h*3600 + m*60 + s);
                return Ok(chrono::Duration::from_std(std)?)
            }
        }
    }
    Err(Error::InvalidDurationFormat)
}

fn parse_datetime (args: &str) -> Result<chrono::NaiveDateTime, chrono::format::ParseError> {
    chrono::NaiveDateTime
        ::parse_from_str(args, "%Y-%m-%d %H:%M:%S")
}


/// Efficient RINEX content decimation
pub fn record_resampling(rnx: &mut Rinex, ops: Vec<(&str, &str)>) {
    for (op, args) in ops.iter() {
        if op.eq(&"time-window") {
            let items = args.split(" ")
                .collect();
            if items.len() == 2 { // date description
                if let Ok(start) = parse_date(items[0].trim()) {
                    if let Ok(end) = parse_date(items[1].trim()) {
                        let start = start.and_hms(0, 0, 0);
                        let end = end.and_hsms(0, 0, 0);
                        rnx.time_window_mut(start, end);
                    } else {
                        println!("failed to parse date from \"{}\" description", items[1]);
                        println!("expecting %Y-%M-%D");
                    }
                } else {
                    println!("failed to parse date from \"{}\" description", items[0]);
                    println!("expecting %Y-%M-%D");
                }
            } else if items.len() == 4 { //datetime description
                let mut start_str = items[0].trim().to_owned();
                start_str.push_str("-");
                start_str.push_str(items[1].trim());
                
                if let Ok(start) = parse_datetime(start_str) {
                    let mut end_str = items[2].trim().to_owned();
                    end_str.push_str("-");
                    end_str.push_str(items[3].trim());
                    if let Ok(end) = parse_datetime(end_str) {
                        rnx.time_window_mut(start, end);
                    }
                } else {
                    println!("failed to parse datetime from \"{}\" description", start_str);
                    println!("expecting %Y-%M-%D-%H:%M%S");
                }
            } else {
                println!("invalid time window description");
                println!("expecting \"%Y-%M-%D - %Y-%M-%D\" or");
                println!("          \"%Y-%M-%D %H:%M%S - %Y-%M-%D %H%M%S\", where");
                println!("first entry is start and last one is end date/datetime descriptor");
            }
            
        } else if op.eq(&"resample-interval") {
            if let Ok(duration) = parse_duration(args.trim()) {
                rnx
                    .decimate_by_interval(duration);
            } else {
                println!("failed to parse chrono::duration from \"{}\"", args);
                println!("Expected format is %HH:%MM:%SS\n");
            }
        } else if op.eq(&"resample-ratio") {
            if let Ok(ratio) = u32::from_str_radix(args.trim(), 10) {
                rnx
                    .decimate_by_ratio_mut(ratio);
            } else {
                println!("failed to parse decimation ratio from \"{}\"", args);
                println!("Expecting unsigned integer value\n");
            }
        }
    }

    Ok(())
}
