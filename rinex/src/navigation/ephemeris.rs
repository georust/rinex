use crate::{
	Sv, Constellation, 
	sv,
	version::Version, 
	Epoch, EpochFlag, 
	epoch::str2date, epoch::ParseDateError,
};
use super::{
	OrbitItem,
	orbits::{
		closest_revision,
		NAV_ORBITS,
	},
};

use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

/// Parsing errors
#[derive(Debug, Error)]
pub enum Error {
	#[error("missing data")]
	MissingData,
	#[error("data base revision error")]
	DataBaseRevisionError,
	#[error("failed to parse data")]
	ParseFloatError(#[from] std::num::ParseFloatError),
	#[error("failed to parse date")]
	ParseDateError(#[from] ParseDateError), 
	#[error("failed to identify sat vehicule")]
	ParseSvError(#[from] sv::Error),
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Ephemeris {
	/// Clock bias [s]
	pub clock_bias: f64,
	/// Clock Drift [s/s]
	pub clock_drift: f64,
	/// Clock Drift Rate [s/s^2]
	pub clock_drift_rate: f64,
	/// Orbits are revision and constellation dependent,
	/// sorted by key and content, described in navigation::database
	pub orbits: HashMap<String, OrbitItem>,
}

impl Default for Ephemeris {
	fn default() -> Self {
		Self {
			clock_bias: 0.0_f64,
			clock_drift: 0.0_f64,
			clock_drift_rate: 0.0_f64,
			orbits: HashMap::new(),
		}
	}
}

impl Ephemeris {
	/// Parses ephemeris from given line iterator
    pub fn parse (mut lines: std::str::Lines<'_>) -> Result<(Epoch, Self), Error> {
		let line = match lines.next() {
			Some(l) => l,
			_ => return Err(Error::MissingData),
		};

		let (svnn, rem) = line.split_at(4);
		let sv = Sv::from_str(svnn.trim())?;
		let (epoch, rem) = rem.split_at(20);
		let epoch = Epoch {
			date: str2date(epoch.trim())?,
			flag: EpochFlag::Ok,
		};

		let (clock_bias, rem) = rem.split_at(19);
		let (clock_dr, clock_drr) = rem.split_at(19);
		let clock_bias = f64::from_str(clock_bias.replace("D","E").trim())?;
		let clock_drift = f64::from_str(clock_dr.replace("D","E").trim())?;
		let clock_drift_rate = f64::from_str(clock_drr.replace("D","E").trim())?;
		let orbits = parse_orbits(
			Version { major: 4, minor: 0 },
			sv.constellation,
			lines)?;

		Ok((epoch,
		Self {
			clock_bias,
			clock_drift,
			clock_drift_rate,
			orbits,
		}))
	}
}

/// Parses constellation + revision dependent orbits data 
fn parse_orbits (version: Version, constell: Constellation, mut lines: std::str::Lines<'_>)
		-> Result<HashMap<String, OrbitItem>, Error> 
{
    // locate closest revision in db
    let db_revision = match closest_revision(constell, version) {
        Some(v) => v,
        _ => return Err(Error::DataBaseRevisionError),
    };

    // retrieve db items / fields to parse
    let items: Vec<_> = NAV_ORBITS
        .iter()
        .filter(|r| r.constellation == constell.to_3_letter_code())
        .map(|r| {
            r.revisions
                .iter()
                .filter(|r| // identified db revision
                    u8::from_str_radix(r.major, 10).unwrap() == db_revision.major
                    && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor
                )
                .map(|r| &r.items)
                .flatten()
        })
        .flatten()
        .collect();

    // parse items
    let mut line = match lines.next() {
        Some(l) => l,
        _ => return Err(Error::MissingData),
    };

    let mut new_line = true;
    let mut total :usize = 0;
    let mut map :HashMap<String, OrbitItem> = HashMap::new();
    for item in items.iter() {
        let (k, v) = item;
        let offset :usize = match new_line {
            false => 19,
            true => {
                new_line = false;
                if version.major == 3 {
                    22+1
                } else {
                    22
                }
            },
        };
        if line.len() >= 19 { // handle empty fields, that might exist..
            let (content, rem) = line.split_at(offset);
            total += offset;
            line = rem.clone();

            if !k.contains(&"spare") { // --> got something to parse in db
                if let Ok(item) = OrbitItem::new(v, content.trim(), constell) {
                    map.insert(k.to_string(), item);
                }
            }

            if total >= 76 {
                new_line = true;
                total = 0;
                if let Some(l) = lines.next() {
                    line = l;
                } else {
                    break
                }
            }
        } else { // early EOL (blank)
            total = 0;
            new_line = true;
            if let Some(l) = lines.next() {
                line = l
            } else {
                break
            }
        }
    }
    Ok(map)
}

