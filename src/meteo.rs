//! `MeteorologicalData` parser and related methods
use std::io::Write;
use thiserror::Error;
use std::str::FromStr;
use std::collections::{BTreeMap, HashMap};
use crate::epoch;
use crate::header;
use crate::header::Header;

#[macro_export]
/// Returns True if 3 letter code
/// matches a temperature observation code
macro_rules! is_temperature_obs_code {
	($code: expr) => {
		$code.eq("TD")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a pressure observation code
macro_rules! is_pressure_obs_code {
	($code: expr) => {
		$code.eq("PR")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a moisture / humidity rate observation code
macro_rules! is_humidity_obs_code {
	($code: expr) => {
		$code.eq("HR")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a wet zenith path delay obs code 
macro_rules! is_wet_zenith_code {
	($code: expr) => {
		$code.eq("ZW")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a dry zenith path delay obs code 
macro_rules! is_dry_zenith_code {
	($code: expr) => {
		$code.eq("ZD")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a wind speed obs code 
macro_rules! is_wind_speed_code {
	($code: expr) => {
		$code.eq("WS")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a rain increment obs code 
macro_rules! is_rain_increment_code {
	($code: expr) => {
		$code.eq("RI")
	};
}

#[macro_export]
/// Returns True if 3 letter code
/// matches a rain increment obs code 
macro_rules! is_hail_indicator_code {
	($code: expr) => {
		$code.eq("HI")
	};
}

/// `Record` content for Meteo data files
pub type Record = BTreeMap<epoch::Epoch, HashMap<String, f32>>;

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum RecordError {
    #[error("failed to parse date")]
    ParseDateError(#[from] epoch::ParseDateError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// Builds `RINEX` record entry for `Meteo` Data files
pub fn build_record_entry (header: &Header, content: &str) 
        -> Result<(epoch::Epoch, HashMap<String, f32>), RecordError> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();

	let mut map : HashMap<String, f32> = HashMap::with_capacity(3);

	// epoch.secs is not f32 as usual
	// Y is 4 digit number as usual for V > 2
	//let (date, rem) = line.split_at(offset);
	let (mut y, m, d, h, min, sec, mut offset) : (i32, u32, u32, u32, u32, u32, usize) 
		= match header.version.major > 2 {
		true => {
			(i32::from_str_radix(line[0..5].trim(),10)?, // Y: 4 digit
			u32::from_str_radix(line[5..8].trim(),10)?, // m
			u32::from_str_radix(line[8..11].trim(),10)?, // d
			u32::from_str_radix(line[11..14].trim(),10)?, // h
			u32::from_str_radix(line[14..17].trim(),10)?, // m
			u32::from_str_radix(line[17..20].trim(),10)?, // s
			20)
		},
		false => {
			(i32::from_str_radix(line[0..3].trim(),10)?, // Y: 2 digit
			u32::from_str_radix(line[3..6].trim(),10)?, // m
			u32::from_str_radix(line[6..9].trim(),10)?,// d
			u32::from_str_radix(line[9..12].trim(),10)?,// h
			u32::from_str_radix(line[12..15].trim(),10)?,// m
			u32::from_str_radix(line[15..18].trim(),10)?,// s
			18)
		},
	};
	if y < 100 { // 2 digit nb case
    	if y > 90 {
        	y += 1900
    	} else {
			y += 2000
		}
	}
	let date = chrono::NaiveDate::from_ymd(y,m,d)
		.and_hms(h,min,sec);
	let flag = epoch::EpochFlag::default();
	let epoch = epoch::Epoch::new(date, flag);

	let obs_codes = &header.met_codes
		.as_ref()
		.unwrap();
	let n_codes = obs_codes.len();
	let nb_lines : usize = num_integer::div_ceil(n_codes, 8).into(); 
	let mut code_index : usize = 0;

	for i in 0..nb_lines {
		for _ in 0..8 {
			let code = &obs_codes[code_index];
			let obs : Option<f32> = match f32::from_str(&line[offset..offset+7].trim()) {
				Ok(f) => Some(f),
				Err(_) => None,
			};

			if let Some(obs) = obs {
				map.insert(code.to_string(), obs); 
			}
			code_index += 1;
			if code_index >= n_codes {
				break
			}

			offset += 7;
			if offset >= line.len() {
				break
			}
		} // 1:8

		if i < nb_lines-1 {
			if let Some(l) = lines.next() {
				line = l;
			} else {
				break
			}
		}
	} // nb lines
	Ok((epoch, map))
}

/// Pushes meteo record into given file writer
pub fn to_file (header: &header::Header, record: &Record, mut writer: std::fs::File) -> std::io::Result<()> {
    let obs_codes = header.met_codes.as_ref().unwrap();
    for epoch in record.keys() {
        write!(writer, " {} ", epoch.date.format("%y %_m %_d %_H %_M %_S").to_string())?;
        for code in obs_codes { 
            write!(writer, "{:.1}   ", record[epoch].get(code).unwrap())?;
        }
        write!(writer, "\n")?
    }
    Ok(())
}
