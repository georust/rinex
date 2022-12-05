use crate::{
    sv,
    epoch, 
    prelude::*,
	version::Version, 
};
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

impl Kepler {
    /// Eearth mass * Gravitationnal field constant [m^3/s^2]
    pub const EARTH_GM_CONSTANT: f64 = 3.986005E14_f64; 
    /// Earth rotation rate in WGS84 frame [rad]
    pub const EARTH_OMEGA_E_WGS84: f64 = 7.292115E-5;
    
    pub const EARTH_A: f64 = 0.0_f64;
    pub const EARTH_A_POW2: f64 = Self::EARTH_A * Self::EARTH_A;

    pub const EARTH_B: f64 = 0.0_f64;
    pub const EARTH_B_POW2: f64 = Self::EARTH_B * Self::EARTH_B;

    pub const EARTH_ECCENTRICITY: f64 = 0.0_f64;
    pub const EARTH_ECCENTRICITY_POW2: f64 = Self::EARTH_ECCENTRICITY * Self::EARTH_ECCENTRICITY;
    pub const EARTH_ECCENTRICITY_POW4: f64 = Self::EARTH_ECCENTRICITY_POW2 * Self::EARTH_ECCENTRICITY_POW2;
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
        self.manual_sat_pos()
    }

    /// Manual calculations of satellite position vector, in ECEF,
    /// from ephemeris
    pub fn manual_sat_pos(&self) -> Option<(f64,f64,f64)> {
        if let Some(kepler) = self.kepler() {
            if let Some(perturbations) = self.perturbations() {
                //TODO
                // double check we always refer to this t0 please
                let t0 = hifitime::GPST_REF_EPOCH; 
                let mut t_k = self.clock_bias - kepler.toe;
                if t_k > 302400.0 {
                    t_k -= 604800.0;
                } else if t_k < -302400.0 {
                    t_k += 604800.0;
                }

                let weeks = match self.get_orbit_f64("gpsWeek") {
                    Some(f) => 0, //f as u32,
                    _ => {
                        match self.get_orbit_f64("galWeek") {
                            Some(f) => 0, //(f as u32) - 1024,
                            _ => 0,
                        }
                    },
                };

                let n_0 = (Kepler::EARTH_GM_CONSTANT /kepler.a.powf(3.0)).sqrt();
                let m_k = kepler.m_0 + (n_0 + perturbations.dn)*t_k; 
                let e_k = m_k + kepler.e * m_k.sin(); //TODO a revoir, recursif

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
                let omega_k = kepler.omega_0 + (perturbations.omega_dot - Kepler::EARTH_OMEGA_E_WGS84)*t_k - Kepler::EARTH_OMEGA_E_WGS84 * kepler.toe; 
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
            let p = (x_k.powf(2.0) + y_k.powf(2.0)).sqrt(); 
            let f = 54.0 * Kepler::EARTH_B.powf(2.0) * z_k.powf(2.0);
            let E2 = Kepler::EARTH_A.powf(2.0) - Kepler::EARTH_B.powf(2.0); 
            let g = p.powf(2.0) + (1.0 - Kepler::EARTH_ECCENTRICITY.powf(2.0)) * z_k.powf(2.0) - Kepler::EARTH_ECCENTRICITY.powf(2.0) * E2;
            let c = Kepler::EARTH_ECCENTRICITY_POW4 * f * p.powf(2.0) / g;
            let s = (1.0 + c + (2.0 * c + c.powf(2.0)).sqrt()).powf(1.0/3.0);
            let pp = f / (s + 1.0/s +1.0).powf(2.0)*3.0*g.powf(2.0);
            let qq = (1.0 + 2.0 * Kepler::EARTH_ECCENTRICITY_POW4 * pp).sqrt();
            let r_0 = 
                (0.5 * Kepler::EARTH_A_POW2 * (1.0 + 1.0/qq)
                    - pp*z_k.powf(2.0)*(1.0 - Kepler::EARTH_ECCENTRICITY_POW2)
                        - 0.5 * pp * p.powf(2.0)).sqrt() - pp * p * Kepler::EARTH_ECCENTRICITY_POW2 / (1.0 + qq);

            let uu = ((p - Kepler::EARTH_ECCENTRICITY_POW2 * r_0).powf(2.0) + z_k.powf(2.0)).sqrt(); 
            let vv = ((p - Kepler::EARTH_ECCENTRICITY_POW2 * r_0).powf(2.0)
                + (1.0 - Kepler::EARTH_ECCENTRICITY_POW2)*z_k.powf(2.0)).sqrt();

            let ep = Kepler::EARTH_A * Kepler::EARTH_ECCENTRICITY / Kepler::EARTH_B;
            let z_0 = Kepler::EARTH_B.powf(2.0) * z_k / Kepler::EARTH_A / vv;

            let lambda = y_k.atan2(x_k);
            let phi = ((z_k + ep.powf(2.0) * z_0) /p).atan();
            let h = uu * (1.0 - Kepler::EARTH_B_POW2 / Kepler::EARTH_A / vv);
            return Some((phi, lambda, h));
        }
        None
    }
    /// Returns (azimuth, elevation, slant range) coordinates in AER system
    /// from parsed orbits and internal calculations
    pub fn angles(&self, ref_pos: (f64,f64,f64)) -> Option<(f64,f64,f64)> {
        if let Some((lat, lon, alt)) = self.sat_latlonalt() {
            let (ref_x, ref_y, ref_z) = ref_pos;
            //TODO revoir si on est tjrs qu'en WGS84,
            //     par exemple le cas de Glonass
            let ref_ellips = map_3d::Ellipsoid::WGS84;
            let (e, n, u) = map_3d::ecef2enu(ref_x, ref_y, ref_z, lat, lon, alt, ref_ellips);
            return Some(map_3d::enu2aer(e, n, u));
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

    let mut key_index: usize = 0;
    let word_size: usize = 19;
    let mut map :HashMap<String, OrbitItem> = HashMap::new();
    for line in lines {
        // trim first few white spaces
        let mut line: &str = match version.major < 3 {
            true =>  &line[3..],
            false => &line[4..],
        };
        let nb_missing = 4 - (line.len() / word_size);
        //println!("LINE \"{}\" | NB MISSING {}", line, nb_missing);
        loop {
            if line.len() == 0 {
                key_index += nb_missing as usize;
                break;
            }
            let (content, rem) = line.split_at(std::cmp::min(word_size, line.len()));
            if let Some((key, token)) = items.get(key_index) {
                //println!("Key \"{}\" | Token \"{}\" | Content \"{}\"", key, token, content.trim()); //DEBUG
                if !key.contains(&"spare") {
                    if let Ok(item) = OrbitItem::new(token, content.trim(), constell) {
                        map.insert(key.to_string(), item);
                    }
                }
            }
            key_index += 1;
            line = rem.clone();
        }
    }
    Ok(map)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn gal_orbit() {
        let content = "
     7.500000000000e+01 1.478125000000e+01 2.945479833915e-09-3.955466341850e-01
     8.065253496170e-07 3.683507675305e-04-3.911554813385e-07 5.440603218079e+03
     3.522000000000e+05-6.519258022308e-08 2.295381450845e+00 7.450580596924e-09
     9.883726443393e-01 3.616875000000e+02 2.551413130998e-01-5.907746081337e-09
     1.839362331110e-10 2.580000000000e+02 2.111000000000e+03                   
     3.120000000000e+00 0.000000000000e+00-1.303851604462e-08 0.000000000000e+00
     3.555400000000e+05";                                                       
        let orbits = parse_orbits(Version::new(3, 0), Constellation::Galileo, content.lines());
        assert!(orbits.is_ok());
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("iodnav"), Some(7.500000000000e+01));
        assert_eq!(ephemeris.get_orbit_f64("crs"), Some(1.478125000000e+01));
        assert_eq!(ephemeris.get_orbit_f64("deltaN"), Some(2.945479833915e-09));
        assert_eq!(ephemeris.get_orbit_f64("m0"), Some(-3.955466341850e-01));
        
        assert_eq!(ephemeris.get_orbit_f64("cuc"), Some(8.065253496170e-07));
        assert_eq!(ephemeris.get_orbit_f64("e"), Some(3.683507675305e-04));
        assert_eq!(ephemeris.get_orbit_f64("cus"), Some(-3.911554813385e-07));
        assert_eq!(ephemeris.get_orbit_f64("sqrta"), Some(5.440603218079e+03));

        assert_eq!(ephemeris.get_orbit_f64("toe"), Some(3.522000000000e+05));
        assert_eq!(ephemeris.get_orbit_f64("cic"), Some(-6.519258022308e-08));
        assert_eq!(ephemeris.get_orbit_f64("omega0"), Some(2.295381450845e+00));
        assert_eq!(ephemeris.get_orbit_f64("cis"), Some(7.450580596924e-09));
      
        assert_eq!(ephemeris.get_orbit_f64("i0"), Some(9.883726443393e-01));
        assert_eq!(ephemeris.get_orbit_f64("crc"), Some(3.616875000000e+02));
        assert_eq!(ephemeris.get_orbit_f64("omega"), Some(2.551413130998e-01));
        assert_eq!(ephemeris.get_orbit_f64("omegaDot"), Some(-5.907746081337e-09));

        assert_eq!(ephemeris.get_orbit_f64("idot"), Some(1.839362331110e-10));
        assert_eq!(ephemeris.get_orbit_f64("dataSrc"), Some(2.580000000000e+02));
        assert_eq!(ephemeris.get_orbit_f64("galWeek"), Some(2.111000000000e+03));
      
        assert_eq!(ephemeris.get_orbit_f64("sisa"), Some(3.120000000000e+00));
        //assert_eq!(ephemeris.get_orbit_f64("health"), Some(0.000000000000e+00));
        assert_eq!(ephemeris.get_orbit_f64("bgdE5aE1"), Some(-1.303851604462e-08));
        assert_eq!(ephemeris.get_orbit_f64("bgdE5bE1"), Some(0.000000000000e+00));

        assert_eq!(ephemeris.get_orbit_f64("t_tm"), Some(3.555400000000e+05));
    }
    #[test]
    fn bds_orbit() {
        let content = "
      .100000000000e+01  .118906250000e+02  .105325815814e-08 -.255139531119e+01
      .169500708580e-06  .401772442274e-03  .292365439236e-04  .649346986580e+04
      .432000000000e+06  .105705112219e-06 -.277512444499e+01 -.211410224438e-06
      .607169709798e-01 -.897671875000e+03  .154887266488e+00 -.871464871438e-10
     -.940753471872e-09  .000000000000e+00  .782000000000e+03  .000000000000e+00
      .200000000000e+01  .000000000000e+00 -.599999994133e-09 -.900000000000e-08
      .432000000000e+06  .000000000000e+00 0.000000000000e+00 0.000000000000e+00";
        let orbits = parse_orbits(Version::new(3, 0), Constellation::BeiDou, content.lines());
        assert!(orbits.is_ok());
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("aode"), Some(1.0));
        assert_eq!(ephemeris.get_orbit_f64("crs"), Some(1.18906250000e+01));
        assert_eq!(ephemeris.get_orbit_f64("deltaN"), Some(0.105325815814e-08));
        assert_eq!(ephemeris.get_orbit_f64("m0"), Some(-0.255139531119e+01));
        
        assert_eq!(ephemeris.get_orbit_f64("cuc"), Some(0.169500708580e-06));
        assert_eq!(ephemeris.get_orbit_f64("e"), Some(0.401772442274e-03));
        assert_eq!(ephemeris.get_orbit_f64("cus"), Some(0.292365439236e-04));
        assert_eq!(ephemeris.get_orbit_f64("sqrta"), Some(0.649346986580e+04));

        assert_eq!(ephemeris.get_orbit_f64("toe"), Some(0.432000000000e+06));
        assert_eq!(ephemeris.get_orbit_f64("cic"), Some(0.105705112219e-06));
        assert_eq!(ephemeris.get_orbit_f64("omega0"), Some(-0.277512444499e+01));
        assert_eq!(ephemeris.get_orbit_f64("cis"), Some(-0.211410224438e-06));
      
        assert_eq!(ephemeris.get_orbit_f64("i0"), Some(0.607169709798e-01));
        assert_eq!(ephemeris.get_orbit_f64("crc"), Some(-0.897671875000e+03));
        assert_eq!(ephemeris.get_orbit_f64("omega"), Some(0.154887266488e+00));
        assert_eq!(ephemeris.get_orbit_f64("omegaDot"), Some(-0.871464871438e-10));

        assert_eq!(ephemeris.get_orbit_f64("idot"), Some(-0.940753471872e-09));
        assert_eq!(ephemeris.get_orbit_f64("bdtWeek"), Some(0.782000000000e+03));
      
        assert_eq!(ephemeris.get_orbit_f64("svAccuracy"), Some(0.200000000000e+01));
        assert_eq!(ephemeris.get_orbit_f64("satH1"), Some(0.0));
        assert_eq!(ephemeris.get_orbit_f64("tgd1b1b3"), Some(-0.599999994133e-09));
        assert_eq!(ephemeris.get_orbit_f64("tgd2b2b3"), Some(-0.900000000000e-08));

        assert_eq!(ephemeris.get_orbit_f64("t_tm"), Some(0.432000000000e+06));
        assert_eq!(ephemeris.get_orbit_f64("aodc"), Some(0.0));
    }
    #[test]
    fn orbits_sat_lat_lon() {
        let content = "
      .100000000000e+01  .118906250000e+02  .105325815814e-08 -.255139531119e+01
      .169500708580e-06  .401772442274e-03  .292365439236e-04  .649346986580e+04
      .432000000000e+06  .105705112219e-06 -.277512444499e+01 -.211410224438e-06
      .607169709798e-01 -.897671875000e+03  .154887266488e+00 -.871464871438e-10
     -.940753471872e-09  .000000000000e+00  .782000000000e+03  .000000000000e+00
      .200000000000e+01  .000000000000e+00 -.599999994133e-09 -.900000000000e-08
      .432000000000e+06  .000000000000e+00 0.000000000000e+00 0.000000000000e+00";
        let orbits = parse_orbits(Version::new(3, 0), Constellation::BeiDou, content.lines());
        assert!(orbits.is_ok());
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: -0.426337239332e-03, 
            clock_drift: -0.752518047875e-10, 
            clock_drift_rate: 0.000000000000e+00,
            orbits,
        };
    }
}
>>>>>>> nav: improve orbit parsing, add tests, introduce position tests
