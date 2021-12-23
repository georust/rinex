//! Navigation.rs
//! to describe `Rinex` file body content
//! for NavigationMessage files
use thiserror::Error;
use chrono::Timelike;

/// `NavigationFrameError` describes
/// navigation frames specific errors
#[derive(Error, Debug)]
pub enum NavigationFrameError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// `NavigationFrame` describes
/// Rinex body frames when
/// Rinex::Header::type::NavigationMessage
#[derive(Debug)]
pub struct NavigationFrame {
    sat_id: u8, // id (PRN#)
    tstamp: chrono::NaiveDateTime, // timestamp
    payload: Vec<f64>, // raw data
}

impl std::str::FromStr for NavigationFrame {
    type Err = NavigationFrameError;
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        let items: Vec<&str> = s.split_ascii_whitespace()
            .collect();
//2.11 (limit?)
/*
 3 20 12 31 23 45  0.0 2.833176404238D-05 0.000000000000D+00 8.637000000000D+04
    1.997111425781D+04 1.119024276733D+00 2.793967723846D-09 0.000000000000D+00
    1.218920263672D+04 8.536128997803D-01 0.000000000000D+00 5.000000000000D+00
   -1.019199707031D+04 3.197331428528D+00 3.725290298462D-09 0.000000000000D+00
*/                      
        let (sat_id_date, remainder) = s.split_at(22);
        let (data1, remainder) = remainder.split_at(19);
        let (data2, remainder) = remainder.split_at(19);
        let (data3, remainder) = remainder.split_at(23);
        let (data4, remainder) = remainder.split_at(19);
        println!("SATIDDATE \"{}\" DATA1 \"{}\" DATA2 \"{}\" 
        DATA3 \"{}\" DATA4\"{}\"", sat_id_date,data1,data2,data3,data4);
        
        let sat_id = u8::from_str_radix(items[0], 10)?;
        let (y,month,day,h,m,s): (i32,u32,u32,u32,u32,f64) =
            (i32::from_str_radix(items[1], 10)?,
            u32::from_str_radix(items[2], 10)?,
            u32::from_str_radix(items[3], 10)?,
            u32::from_str_radix(items[4], 10)?,
            u32::from_str_radix(items[5], 10)?,
            f64::from_str(items[6])?);
        let tstamp = chrono::NaiveDate::from_ymd(y,month,day)
            .and_hms(h,m,s as u32);
        let mut payload: Vec<f64> = Vec::new();
        for i in 7..items.len() {
            payload.push(f64::from_str(items[i])?)
        }
        Ok(NavigationFrame {
            sat_id,
            tstamp,
            payload,
        })

    }
}
