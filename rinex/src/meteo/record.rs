use thiserror::Error;
use std::str::FromStr;
use std::collections::{BTreeMap, HashMap};
use crate::{
    Epoch, EpochFlag,
    epoch::{
        str2date, ParseDateError,
    },
    version,
    Header,
    merge, merge::Merge,
    split, split::Split,
};
use super::observable::Observable;

/// Meteo RINEX Record content. 
/// Data is sorted by [epoch::Epoch] and by Observable.
/// Example of Meteo record browsing and manipulation
/// ```
/// use rinex::*;
/// // grab a METEO RINEX
/// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
///    .unwrap();
/// // grab record
/// let record = rnx.record.as_meteo()
///    .unwrap();
/// for (epoch, observables) in record.iter() {
///    for (observable, data) in observables.iter() {
///        // Observable is standard 3 letter string code
///        // Data is raw floating point data
///    }
/// }
/// ```
pub type Record = BTreeMap<Epoch, HashMap<Observable, f64>>;

/// Returns true if given line matches a new Meteo Record `epoch`
pub fn is_new_epoch (line: &str, v: version::Version) -> bool {
    if v.major < 4 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() { // minimum epoch descriptor 
            return false
        }
        let datestr = &line[1..min_len.len()]; 
        str2date(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() { // minimum epoch descriptor
            return false
        }
        let datestr = &line[1..min_len.len()]; 
        str2date(datestr).is_ok() // valid epoch descriptor
    }
}

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse date")]
    ParseDateError(#[from] ParseDateError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// Builds `Record` entry for `MeteoData`
pub fn parse_epoch (header: &Header, content: &str) 
        -> Result<(Epoch, HashMap<Observable, f64>), Error> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();

	let mut map : HashMap<Observable, f64> = HashMap::with_capacity(3);

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
	let flag = EpochFlag::default();
	let epoch = Epoch::new(date, flag);

	let codes = &header.meteo
        .as_ref()
        .unwrap()
        .codes;
	let n_codes = codes.len();
	let nb_lines : usize = num_integer::div_ceil(n_codes, 8).into(); 
	let mut code_index : usize = 0;

	for i in 0..nb_lines {
		for _ in 0..8 {
			let code = &codes[code_index];
			let obs : Option<f64> = match f64::from_str(&line[offset..offset+7].trim()) {
				Ok(f) => Some(f),
				Err(_) => None,
			};

			if let Some(obs) = obs {
				map.insert(code.clone(), obs); 
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

/// Writes epoch into given streamer
pub fn fmt_epoch (
        epoch: &Epoch, 
        data: &HashMap<Observable, f64>, 
        header: &Header, 
    ) -> Result<String, Error>  {
    let mut lines = String::with_capacity(128);
    if header.version.major > 3 {
        lines.push_str(&format!(" {}", 
            epoch.date.format("%Y %_m %_d %_H %_M %_S").to_string()));
    } else {
        lines.push_str(&format!(" {}", 
            epoch.date.format("%y %_m %_d %_H %_M %_S").to_string()));
    }
    let observables = &header
        .meteo
        .as_ref()
        .unwrap()
        .codes;
    let mut index = 0;
    for obscode in observables {
        index += 1;
        if let Some(data) = data.get(obscode) {
            lines.push_str(&format!("{:7.1}", data));
        } else {
            lines.push_str(&format!("       "));
        }
        if (index %8) == 0 {
            lines.push_str("\n");
        }
    }
    lines.push_str("\n");
    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert_eq!(is_new_epoch(content, 
            version::Version {
                major: 2,
                minor: 0,
            }), true);
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert_eq!(is_new_epoch(content,
            version::Version {
                major: 4,
                minor: 0,
            }), true);
    }
}

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, observations) in rhs.iter() {
            if let Some(oobservations) = self.get_mut(epoch) {
                for (observation, data) in observations.iter() {
                    if !oobservations.contains_key(observation) { // new observation
                        oobservations.insert(observation.clone(), *data);
                    }
                }
            } else { // new epoch
                self.insert(*epoch, observations.clone());
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self.iter()
            .flat_map(|(k, v)| {
                if k.date < epoch.date {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self.iter()
            .flat_map(|(k, v)| {
                if k.date >= epoch.date {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
}
