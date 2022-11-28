use crate::{
	Sv, Constellation, 
	sv,
	version::Version, 
    epoch, 
	Epoch,
};
use rust_3d::Point3D;
use super::{
    orbits::{closest_revision, NAV_ORBITS},
    OrbitItem,
};
use crate::{epoch, sv, version::Version, Constellation, Epoch, Sv};

use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

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
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
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
#[derive(Clone, Debug, PartialEq)]
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

/// Eearth mass * Gravitationnal field constant [m^3/s^2]
pub const EARTH_GM_CONSTANT: f64 = 3.986005E14_f64; 
/// Earth rotation rate in WGS84 frame [rad]
pub const EARTH_ROT_RATE_RAD_WGS64: f64 = 7.292115E-5;

/*
/// Macro to convert Sv time to UTC, correction
/// is GNSS dependent
pub fn sv_t_utc(epoch: Epoch,
*/

/// Kepler parameters
pub struct Kepler {
    /// sqrt(semi major axis) [sqrt(m)]
    pub sqrt_a: f64,
    /// Eccentricity [n.a]
    pub e: f64,
    /// Inclination angle at reference time [semicircles]
    pub i_0: f64,
    /// Longitude of ascending node at reference time [semicircles]
    pub omega_0: f64,
    /// Mean anomaly at reference time [semicircles]
    pub m_0: f64,
    /// argument of perigee [semicircles]
    pub omega: f64,
    // /// internal buffer for E_k linear solving
    // buffer: Vec<f64>,
}

impl Kepler {
    /// Computes t_k
    pub fn t_k(&self) -> f64 {
        0.0
    }
    /// Returns M_k term
    #[allow(non_snake_case)]
    pub fn M_k(&mut self) -> f64 {
        0.0
    }
    /// Returns E_k term
    #[allow(non_snake_case)]
    pub fn E_k(&mut self) -> f64 {
        0.0
    }
    /// Returns r_k term
    pub fn r_k(&self) -> f64 {
        //self.sqrt_a.powf(2.0)*(1.0 - self.e * 
        0.0
    }
}

/// Perturbation parameters
pub struct Perturbations {
    /// Mean motion difference from computed value [semicircles.s-1]
    pub dn: f64,
    /// Inclination rate of change [semicircles.s-1]
    pub i_dot: f64,
    /// Right ascension rate of change [semicircles.s^-1]
    pub omega_dot: f64,
    /// Amplitude of sine harmonic correction term of the argument
    /// of latitude [rad]
    pub cus: f64,
    /// Amplitude of cosine harmonic correction term of the argument
    /// of latitude [rad]
    pub cuc: f64,
    /// Amplitude of sine harmonic correction term of the angle of inclination [rad]
    pub cis: f64,
    /// Amplitude of cosine harmonic correction term of the angle of inclination [rad]
    pub cic: f64,
    /// Amplitude of sine harmonic correction term of the orbit radius [m]
    pub crs: f64,
    /// Amplitude of cosine harmonic correction term of the orbit radius [m]
    pub crc: f64,
}

impl Ephemeris {
    /// Retrieve all clock biases (bias, drift, drift_rate) at once
    pub fn clock_data(&self) -> (f64,f64,f64) {
        (self.clock_bias, self.clock_drift, self.clock_drift_rate)
    }

    /// Retrieves orbit data as f64 value, if possible
    pub fn get_orbit_f64(&self, field: &str) -> Option<f64> {
        if let Some(v) = self.orbits.get(field) {
            v.as_f64()
        } else {
            None
        }
    }

    /// Retrieve Keplerian parameters
    pub fn kepler(&self) -> Option<Kepler> {
        if let Some(sqrt_a) = self.get_orbit_f64("sqrta") {
            if let Some(e) = self.get_orbit_f64("e") {
                if let Some(i_0) = self.get_orbit_f64("i0") {
                    if let Some(omega_0) = self.get_orbit_f64("omega0") {
                        if let Some(m_0) = self.get_orbit_f64("m0") {
                            if let Some(omega) = self.get_orbit_f64("omega") {
                                return Some(Kepler {
                                    e,
                                    sqrt_a,
                                    i_0,
                                    omega_0,
                                    m_0,
                                    omega,
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
	
    /// Retrieve Perturbation parameters
    pub fn perturbations(&self) -> Option<Perturbations> {
        if let Some(cuc) = self.get_orbit_f64("cuc") {
            if let Some(cus) = self.get_orbit_f64("cus") {
                if let Some(cis) = self.get_orbit_f64("cis") {
                    if let Some(cic) = self.get_orbit_f64("cic") {
                        if let Some(crs) = self.get_orbit_f64("crs") {
                            if let Some(crc) = self.get_orbit_f64("crc") {
                                if let Some(dn) = self.get_orbit_f64("deltaN") {
                                    if let Some(i_dot) = self.get_orbit_f64("idot") {
                                        if let Some(omega_dot) = self.get_orbit_f64("omegaDot") {
                                            return Some(Perturbations {
                                                dn,
                                                i_dot,
                                                omega_dot,
                                                cuc,
                                                cus,
                                                cic,
                                                cis,
                                                crc,
                                                crs,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Returns satellite position triplet, in ecef,
    /// if such information is contained in parsed orbits
    pub fn sat_pos(&self) -> Option<(f64,f64,f64)> {
        if let Some(pos_x) = self.get_orbit_f64("satPosX") {
            if let Some(pos_y) = self.get_orbit_f64("satPosY") {
                if let Some(pos_z) = self.get_orbit_f64("satPosZ") { 
                    return Some((pos_x,pos_y,pos_z));
                }
            }
        }
        None
    }

    /// Computes elevation angle. Useful macro for the user
    /// to retrieve the elevation angle from this Ephemeris frame (given epoch),
    /// withouth further data identification.
    /// Station position (ECEF(x,y,z)) is a requirement.
    /// This information is provided in the file header.
    pub fn elevation_angle(&self, station_ecef: &Option<Point3D>) -> Option<f64> {
        None
    }

    /// Parses ephemeris from given line iterator
    pub fn parse_v2v3(
        version: Version,
        constellation: Constellation,
        mut lines: std::str::Lines<'_>,
    ) -> Result<(Epoch, Sv, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::MissingData),
        };

        let svnn_offset: usize = match version.major < 3 {
            true => 3,
            false => 4,
        };
        let date_offset: usize = match version.major < 3 {
            true => 19,
            false => 19,
        };

        let (svnn, rem) = line.split_at(svnn_offset);
        let (date, rem) = rem.split_at(date_offset);
        let (epoch, _) = epoch::parse(date.trim())?;
        let (clk_bias, rem) = rem.split_at(19);
        let (clk_dr, clk_drr) = rem.split_at(19);

        let sv: Sv = match version.major {
            1 | 2 => {
                match constellation {
                    Constellation::Mixed => {
                        // not sure that even exists
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

        let clock_bias = f64::from_str(clk_bias.replace("D", "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace("D", "E").trim())?;
        let clock_drift_rate = f64::from_str(clk_drr.replace("D", "E").trim())?;
        let orbits = parse_orbits(version, sv.constellation, lines)?;
        Ok((
            epoch,
            sv,
            Self {
                clock_bias,
                clock_drift,
                clock_drift_rate,
                orbits,
            },
        ))
    }

    pub fn parse_v4(mut lines: std::str::Lines<'_>) -> Result<(Epoch, Sv, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::MissingData),
        };

        let (svnn, rem) = line.split_at(4);
        let sv = Sv::from_str(svnn.trim())?;
        let (epoch, rem) = rem.split_at(19);
        let (epoch, _) = epoch::parse(epoch.trim())?;

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
}

/// Parses constellation + revision dependent orbits data
fn parse_orbits(
    version: Version,
    constell: Constellation,
    mut lines: std::str::Lines<'_>,
) -> Result<HashMap<String, OrbitItem>, Error> {
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
                    && u8::from_str_radix(r.minor, 10).unwrap() == db_revision.minor)
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
    let mut total: usize = 0;
    let mut map: HashMap<String, OrbitItem> = HashMap::new();
    for item in items.iter() {
        let (k, v) = item;
        let offset: usize = match new_line {
            false => 19,
            true => {
                new_line = false;
                if version.major == 3 {
                    22 + 1
                } else {
                    22
                }
            },
        };
        if line.len() >= 19 {
            // handle empty fields, that might exist..
            let (content, rem) = line.split_at(offset);
            total += offset;
            line = rem.clone();

            if !k.contains(&"spare") {
                // --> got something to parse in db
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
                    break;
                }
            }
        } else {
            // early EOL (blank)
            total = 0;
            new_line = true;
            if let Some(l) = lines.next() {
                line = l
            } else {
                break;
            }
        }
    }
    Ok(map)
}
