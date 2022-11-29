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

/// Kepler parameters
pub struct Kepler {
    /// sqrt(semi major axis) [sqrt(m)]
    pub a: f64,
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
    /// time of issue of ephemeris
    pub toe: f64,
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
                                if let Some(toe) = self.get_orbit_f64("toe") {
                                    return Some(Kepler {
                                        a: sqrt_a.powf(2.0),
                                        e,
                                        i_0,
                                        omega_0,
                                        m_0,
                                        omega,
                                        toe,
                                    });
                                }
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

    /// Returns satellite position vector, in ECEF
    pub fn sat_pos(&self) -> Option<(f64,f64,f64)> {
        if let Some(pos_x) = self.get_orbit_f64("satPosX") {
            if let Some(pos_y) = self.get_orbit_f64("satPosY") {
                if let Some(pos_z) = self.get_orbit_f64("satPosZ") { 
                    return Some((pos_x,pos_y,pos_z));
                }
            }
        }
        manual_sat_pos()
    }

    /// Manual calculations of satellite position vector, in ECEF,
    /// from ephemeris
    pub fn manual_sat_pos(&self) -> Option<(f64,f64,f64)> {
        if let Some(kepler) = self.kepler() {
            if let Some(perturbations) = self.perturbations() {
                let mut t_k = self.clock_bias - kepler.toe;
                if t_k > 302400 {
                    t_k -= 604800;
                } else if t_k < -302400 {
                    t_k += 604800;
                }
                let n_0 = (GM/kepler.a.powf(3.0)).sqrt();
                let m_k = kepler.m_0 + (n_0 + perturbations.dn)*t_k; 

                let e_k = 0.0_f64; //TODO
                let v_k = (((1.0-kepler.e.powf(2.0)).sqrt() * e_k.sin()) / (e_k.cos() - kepler.e)).atan();
                let phi_k = v_k + kepler.omega;
                let du_k = perturbations.cus * (2.0*phi_k).sin() + perturbations.cuc * (2.0*phi_k).cos();
                let u_k = phi_k + du_k;
                let dr_k = perturbations.crs * (2.0*phi_k).sin() + perturbations.crc * (2.0*phi_k).cos();
                let r_k = kepler.a * (1.0 - kepler.e * e_k.cos()) + dr_k;

                let di_k = perturbations.cis * (2.0*phi_k).sin() + perturbations.cic * (2.0*phi_k).cos();
                let i_k = kepler.i_0 + di_k + perturbations.i_dot * t_k;

                let xp_k = r_k * u_k.cos();
                let yp_k = r_k * u_k.sin();
                let omega_k = kepler.omega_0 + (perturbations.omega_dot - OMEGA_E)*t_k - OMEGA_E * kepler.toe; 
                let x_k = xp_k * omega_k.cos() - yp_k * omega_k.sin() * i_k.cos();
                let y_k = xp_k * omega_k.sin() + yp_k * omega_k.cos() * i_k.cos();
                let z_k = yp_k * i_k.sin();
                return Some((x_k, y_k, z_k));
            }
        }
        None
    }
    /// Computes satellite position in (latitude, longitude, altitude)
    pub fn sat_latlonalt(&self) -> Option<(f64,f64,f64)> {
        if let Some((x_k, y_k, z_k)) = self.sat_pos() {
            let p = (x_k.pow(2.0) + y_k.pow(2.0)).sqrt(); 
            let f = 54.0 * EARTH_B.pow(2.0) * z_k.pow(2.0);
            //todo EARTH_ECCENTRICTY_POW4 CONST
            //TODO CONST EARTH_A EARTH_B
            let E2 = EARTH_A.pow(2.0) - EARTH_B.pow(2.0); 
            let g = p.pow(2.0) + (1.0 - EARTH_ECCENTRICITY.pow(2.0)) * z_k.pow(2.0) - EARTH_ECCENTRICITY.pow(2.0) * E2;
            let c = EARTH_ECCENTRICITY_POW4 * f * p.pow(2.0) / g;
            let s = (1.0 + c + (2.0 * c + c.pow(2.0)).sqrt()).pow(1.0/3.0);
            let pp = f / (s + 1/s +1).pow(2.0)*3.0*g.pow(2.0);
            let qq = (1.0 + 2.0 * EARTH_ECCENTRICITY_POW4 * pp).sqrt();
            let r_0 = 
                (0.5 * EARTH_A_POW2 * (1.0 + 1.0/qq)
                    - pp*z_k.pow(2.0)*(1.0 - EARTH_ECCENTRICY_POW2)
                        - 0.5 * pp * p.pow(2.0)).sqrt() - pp * p * EARTH_ECCENTRICY_POW2 / (1.0 + qq);

            let uu = ((p - EARTH_ECCENTRICITY_POW2 * r_0).pow(2.0) + z_k.pow(2.0)).sqrt(); 
            let vv = ((p - EARTH_ECCENTRICY_POW2 * r_0).pow(2.0)
                + (1.0 - EARTH_ECCENTRICITY_POW2)*z_k.pow(2.0)).sqrt();

            //TODO CONST
            let ep = EARTH_A * EARTH_ECCENTRICITY / EARTH_B;
            let z_0 = EARTH_B.pow(2.0) * z_k / EARTH_A / vv;

            let lambda = (y_k / x_k).atan2();
            let phi = ((z_k + ep.pow(2.0) * z_0) /p).atan();
            let h = uu * (1.0 - EARTH_B_POW2 / EARTH_A / vv);
            return Some((phi, lambda, h));
        }
        None
    }
    /// Computes (azimuth, elevation) angles. Useful macro for the user
    /// to retrieve the elevation angle from this Ephemeris frame (given epoch),
    pub fn angles(&self) -> Option<(f64,f64)> {
        if let Some((phi, lambda, h)) = self.sat_latlonalt() {
            let theta_i = 
        }
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
