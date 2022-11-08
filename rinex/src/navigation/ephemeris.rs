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
	#[error("failed to parse data")]
	ParseIntError(#[from] std::num::ParseIntError),
	#[error("failed to parse date")]
	ParseDateError(#[from] ParseDateError), 
	#[error("failed to identify sat vehicule")]
	ParseSvError(#[from] sv::Error),
}

/// Most common Navigation Frame content.
/// ```
/// use rinex::prelude::*;
/// use rinex::navigation::*;
/// let rnx = Rinex::from_file("../test_resources/NAV/V2/amel0010.21g")
///     .unwrap();
/// let record = rnx.record.as_nav()
///     .unwrap();
/// for (epoch, classes) in record {
///     for (class, frames) in classes {
///         for fr in frames {
///             let (msg_type, sv, ephemeris) = fr.as_eph()
///                 .unwrap(); // Until RINEX4, Ephemeris frames were the only
///                     // existing frames. So you're fine with this naive assumption
///             assert_eq!(*msg_type, MsgType::LNAV); // LNAV for Legacy NAV
///                 // is the only existing value up until RINEX4.
///                 // msg_type is truly only exploited in RINEX4 anyways.
///             // Ephemeris come with Sv embedded clock estimates
///             // Sv Clock offset, or clock bias [s]
///             // Sv Clock Drift (d(offset)/dt) [s.s¯¹]
///             // Sv Clock Drift Rate (d(drift)/dt) [s.s¯²]
///             let (clk_offset, clk_drift, clk_drift_r) = ephemeris.clock_data();
///             // we provide some macros for standard fields
///             if let Some(elev) = ephemeris.elevation_angle() {
///                 // not all ephemeris come with this important information
///             }
///             // Orbits are then revision & constellation dependent
///             let orbits = &ephemeris.orbits;
///             for (field, data) in orbits {
///                 // Field is a high level description.
///                 // Data is an [navigation::OrbitItem]
///                 // Most data is to be interprated as floating point value
///                 if field.eq("satPosX") {
///                     let x = data.as_f64();
///                 }
///                 // Some high level enums exist, like Sv Health enums
///                 if field.eq("health") {
///                     if let Some(h) = data.as_glo_health() {
///                         assert_eq!(h, GloHealth::Healthy);
///                     }
///                 }
///                 // Another way to do this would be:
///                 if let Some(h) = data.as_glo_health() {
///                     assert_eq!(h, GloHealth::Healthy);
///                 }
///             }
///             // index orbits directly, if you know what you're doing
///             if let Some(x) = orbits.get("satPosX") {
///                 if let Some(x) = x.as_f64() {
///                     // these are Glonass Orbit Fields
///                     let x = x.sqrt();
///                 }
///             }
///         }
///     }
/// }
/// ```
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
    /// Retrieve all clock fields (bias, drift, drift_rate) at once
    pub fn clock_data(&self) -> (f64,f64,f64) {
        (self.clock_bias, self.clock_drift, self.clock_drift_rate)
    }
	/// Parses ephemeris from given line iterator
    pub fn parse_v2v3 (version: Version, constellation: Constellation, mut lines: std::str::Lines<'_>) -> Result<(Epoch, Sv, Self), Error> {
		let line = match lines.next() {
			Some(l) => l,
			_ => return Err(Error::MissingData), 
		};
		
		let svnn_offset: usize = match version.major {
			1|2 => 2, // Y
			3 => 4, // XYY
			_ => unreachable!(),
		};

		let (svnn, rem) = line.split_at(svnn_offset);
		let (date, rem) = rem.split_at(20);
		let epoch = Epoch {
			date: str2date(date.trim())?,
			flag: EpochFlag::default(),
		};

		let (clk_bias, rem) = rem.split_at(19);
		let (clk_dr, clk_drr) = rem.split_at(19);

		let sv : Sv = match version.major {
			1|2 => {
				match constellation {
					Constellation::Mixed => { // not sure that even exists
						Sv::from_str(svnn.trim())?
					},
					_ => {
						Sv {
							constellation, // constellation.clone(),
							prn: u8::from_str_radix(svnn.trim(), 10)?,
						}
					},
				}
			},
			3 => Sv::from_str(svnn.trim())?,
			_ => unreachable!(),
		};

		let clock_bias = f64::from_str(clk_bias.replace("D","E").trim())?;
		let clock_drift = f64::from_str(clk_dr.replace("D","E").trim())?;
		let clock_drift_rate = f64::from_str(clk_drr.replace("D","E").trim())?;
		let orbits = parse_orbits(version, sv.constellation, lines)?;
		Ok((epoch,
			sv,
			Self {
				clock_bias,
				clock_drift,
				clock_drift_rate,
				orbits,
			},
		))
	}

	pub fn parse_v4 (mut lines: std::str::Lines<'_>) -> Result<(Epoch, Sv, Self), Error> {
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

		let (clk_bias, rem) = rem.split_at(19);
		let (clk_dr, clk_drr) = rem.split_at(19);
		let clock_bias = f64::from_str(clk_bias.replace("D","E").trim())?;
		let clock_drift = f64::from_str(clk_dr.replace("D","E").trim())?;
		let clock_drift_rate = f64::from_str(clk_drr.replace("D","E").trim())?;
		let orbits = parse_orbits(
			Version { major: 4, minor: 0 },
			sv.constellation,
			lines)?;
		Ok((epoch, 
			sv,
			Self {
				clock_bias,
				clock_drift,
				clock_drift_rate,
				orbits,
			},
		))
	}

    /// Computes elevation angle. Useful macro so the user
    /// does not have to either care for the Orbit field identification,
    /// or involved computations
    pub fn elevation_angle(&self) -> Option<f64> {
        if let Some(e) = self.orbits.get("e") {
            e.as_f64()
        } else {
            // Orbit field was either missing
            // but what about glonass ??
            None
        }
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

