use super::{
    orbits::{closest_revision, NAV_ORBITS},
    OrbitItem,
};
use crate::{epoch, prelude::*, sv, version::Version};
use hifitime::{Unit, GPST_REF_EPOCH};
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
#[derive(Clone, Debug)]
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
    pub const EARTH_GM_CONSTANT: f64 = 3.986004418E14_f64;
    /// Earth rotation rate in WGS84 frame [rad]
    pub const EARTH_OMEGA_E_WGS84: f64 = 7.2921151467E-5;

    pub const EARTH_A: f64 = 0.0_f64;
    pub const EARTH_A_POW2: f64 = Self::EARTH_A * Self::EARTH_A;

    pub const EARTH_B: f64 = 0.0_f64;
    pub const EARTH_B_POW2: f64 = Self::EARTH_B * Self::EARTH_B;

    pub const EARTH_ECCENTRICITY: f64 = 0.0_f64;
    pub const EARTH_ECCENTRICITY_POW2: f64 = Self::EARTH_ECCENTRICITY * Self::EARTH_ECCENTRICITY;
    pub const EARTH_ECCENTRICITY_POW4: f64 =
        Self::EARTH_ECCENTRICITY_POW2 * Self::EARTH_ECCENTRICITY_POW2;
}

/// Perturbation parameters
#[derive(Clone, Debug)]
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
    pub fn clock_data(&self) -> (f64, f64, f64) {
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

    /// Tries to retrieve week counter
    pub fn get_weeks(&self) -> Option<u32> {
        //TODO:
        // cast/scalings per constellation ??
        if let Some(v) = self.orbits.get("gpsWeek") {
            if let Some(f) = v.as_f64() {
                return Some(f.round() as u32);
            }
        } else if let Some(v) = self.orbits.get("bdtWeek") {
            if let Some(f) = v.as_f64() {
                return Some(f.round() as u32);
            }
        } else if let Some(v) = self.orbits.get("galWeek") {
            if let Some(f) = v.as_f64() {
                return Some(f.round() as u32);
            }
        }
        None
    }

    /// Retrieves Keplerian parameters
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

    /// Retrieves Perturbations
    pub fn perturbations(&self) -> Option<Perturbations> {
        if let Some(cuc) = self.get_orbit_f64("cuc") {
            if let Some(cus) = self.get_orbit_f64("cus") {
                if let Some(cic) = self.get_orbit_f64("cic") {
                    if let Some(cis) = self.get_orbit_f64("cis") {
                        if let Some(crc) = self.get_orbit_f64("crc") {
                            if let Some(crs) = self.get_orbit_f64("crs") {
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
    pub fn sat_pos_ecef(&self, epoch: Epoch) -> Option<(f64, f64, f64)> {
        if let Some(pos_x) = self.get_orbit_f64("satPosX") {
            if let Some(pos_y) = self.get_orbit_f64("satPosY") {
                if let Some(pos_z) = self.get_orbit_f64("satPosZ") {
                    //TODO PZ90
                    //     add check for SBAS
                    return Some((pos_x, pos_y, pos_z));
                }
            }
        }
        self.kepler2ecef(epoch)
    }

    /// Returns satellite instantaneous speed estimate, in ECEF
    pub fn sat_speed_ecef(
        &self,
        epoch: Epoch,
        prev_pos: (f64, f64, f64),
        prev_epoch: Epoch,
    ) -> Option<(f64, f64, f64)> {
        if let Some(vel_x) = self.get_orbit_f64("velX") {
            if let Some(vel_y) = self.get_orbit_f64("velY") {
                if let Some(vel_z) = self.get_orbit_f64("velZ") {
                    //TODO PZ90
                    //     add check for SBAS
                    return Some((vel_x, vel_y, vel_z));
                }
            }
        }
        if let Some((pos_x, pos_y, pos_z)) = self.kepler2ecef(epoch) {
            let dt = (epoch - prev_epoch).to_unit(Unit::Second);
            let dx = (pos_x - prev_pos.0) / dt;
            let dy = (pos_y - prev_pos.1) / dt;
            let dz = (pos_z - prev_pos.2) / dt;
            return Some((dx, dy, dz));
        }
        None
    }

    /// Returns satellite instantaneous acelleration estimate, in ECEF
    pub fn sat_accel_ecef(
        &self,
        epoch: Epoch,
        prev_pos: (f64, f64, f64),
        prev_speed: (f64, f64, f64),
        prev_epoch: Epoch,
    ) -> Option<(f64, f64, f64)> {
        if let Some(accel_x) = self.get_orbit_f64("accelX") {
            if let Some(accel_y) = self.get_orbit_f64("accelY") {
                if let Some(accel_z) = self.get_orbit_f64("accelZ") {
                    //TODO PZ90
                    //     add check for SBAS
                    return Some((accel_x, accel_y, accel_z));
                }
            }
        }
        if let Some((dx, dy, dz)) = self.sat_speed_ecef(epoch, prev_speed, prev_epoch) {
            let dt = (epoch - prev_epoch).to_unit(Unit::Second);
            let ddx = (dx - prev_speed.0) / dt;
            let ddy = (dy - prev_speed.1) / dt;
            let ddz = (dz - prev_speed.2) / dt;
            return Some((ddx, ddy, ddz));
        }
        None
    }

    /// Manual calculations of satellite position vector, in ECEF.
    /// `epoch`: orbit epoch
    pub fn kepler2ecef(&self, epoch: Epoch) -> Option<(f64, f64, f64)> {
        let kepler = self.kepler()?;
        let perturbations = self.perturbations()?;

        //TODO: double check we always refer to this t_0
        let weeks = self.get_weeks()?;
        let t0 = GPST_REF_EPOCH + Duration::from_days((weeks * 7).into());
        let toe = t0 + Duration::from_seconds(kepler.toe as f64);
        let t_k = (epoch - toe).to_seconds();

        let n0 = (Kepler::EARTH_GM_CONSTANT / kepler.a.powf(3.0)).sqrt();
        let n = n0 + perturbations.dn;
        let m_k = kepler.m_0 + n * t_k;
        let e_k = m_k + kepler.e * m_k.sin();
        let nu_k = ((1.0 - kepler.e.powf(2.0)).sqrt() * e_k.sin()).atan2(e_k.cos() - kepler.e);
        let phi_k = nu_k + kepler.omega;

        let du_k =
            perturbations.cuc * (2.0 * phi_k).cos() + perturbations.cus * (2.0 * phi_k).sin();
        let u_k = phi_k + du_k;

        let di_k =
            perturbations.cic * (2.0 * phi_k).cos() + perturbations.cis * (2.0 * phi_k).sin();
        let i_k = kepler.i_0 + perturbations.i_dot * t_k + di_k;

        let dr_k =
            perturbations.crc * (2.0 * phi_k).cos() + perturbations.crs * (2.0 * phi_k).sin();
        let r_k = kepler.a * (1.0 - kepler.e * e_k.cos()) + dr_k;

        let omega_k = kepler.omega_0
            + (perturbations.omega_dot - Kepler::EARTH_OMEGA_E_WGS84) * t_k
            - Kepler::EARTH_OMEGA_E_WGS84 * kepler.toe;
        let xp_k = r_k * u_k.cos();
        let yp_k = r_k * u_k.sin();

        let x_k = xp_k * omega_k.cos() - yp_k * omega_k.sin() * i_k.cos();
        let y_k = xp_k * omega_k.sin() + yp_k * omega_k.cos() * i_k.cos();
        let z_k = yp_k * i_k.sin();

        Some((x_k, y_k, z_k))
    }

    /// Computes satellite position in (latitude, longitude, altitude)
    pub fn sat_latlonalt(&self, epoch: Epoch) -> Option<(f64, f64, f64)> {
        let (x_k, y_k, z_k) = self.sat_pos_ecef(epoch)?;
        Some(map_3d::ecef2geodetic(
            x_k,
            y_k,
            z_k,
            map_3d::Ellipsoid::WGS84,
        ))
    }

    /// Computes and returns vehicle (azimuth, elevation) angles, in degrees
    pub fn sat_angles(&self, epoch: Epoch, ref_pos: (f64, f64, f64)) -> Option<(f64, f64)> {
        let (lat, lon, alt) = self.sat_latlonalt(epoch)?;
        let (ref_x, ref_y, ref_z) = ref_pos;
        let ref_ellips = map_3d::Ellipsoid::WGS84;
        let (e, n, u) = map_3d::ecef2enu(ref_x, ref_y, ref_z, lat, lon, alt, ref_ellips);
        let aer = map_3d::enu2aer(e, n, u);
        Some((aer.0, aer.1))
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
        let clock_bias = f64::from_str(clk_bias.replace("D", "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace("D", "E").trim())?;
        let clock_drift_rate = f64::from_str(clk_drr.replace("D", "E").trim())?;
        let orbits = parse_orbits(Version { major: 4, minor: 0 }, sv.constellation, lines)?;
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
    let mut map: HashMap<String, OrbitItem> = HashMap::new();
    for line in lines {
        // trim first few white spaces
        let mut line: &str = match version.major < 3 {
            true => &line[3..],
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
    fn build_orbits(
        constellation: Constellation,
        descriptor: Vec<(&str, &str)>,
    ) -> HashMap<String, OrbitItem> {
        let mut map: HashMap<String, OrbitItem> = HashMap::with_capacity(descriptor.len());
        for (key, value) in descriptor.iter() {
            map.insert(
                key.to_string(),
                OrbitItem::new("f64", value, constellation).unwrap(),
            );
        }
        map
    }
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
        assert_eq!(
            ephemeris.get_orbit_f64("omegaDot"),
            Some(-5.907746081337e-09)
        );

        assert_eq!(ephemeris.get_orbit_f64("idot"), Some(1.839362331110e-10));
        assert_eq!(ephemeris.get_orbit_f64("dataSrc"), Some(2.580000000000e+02));
        assert_eq!(ephemeris.get_weeks(), Some(2111));

        assert_eq!(ephemeris.get_orbit_f64("sisa"), Some(3.120000000000e+00));
        //assert_eq!(ephemeris.get_orbit_f64("health"), Some(0.000000000000e+00));
        assert_eq!(
            ephemeris.get_orbit_f64("bgdE5aE1"),
            Some(-1.303851604462e-08)
        );
        assert_eq!(
            ephemeris.get_orbit_f64("bgdE5bE1"),
            Some(0.000000000000e+00)
        );

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
        assert_eq!(
            ephemeris.get_orbit_f64("omegaDot"),
            Some(-0.871464871438e-10)
        );

        assert_eq!(ephemeris.get_orbit_f64("idot"), Some(-0.940753471872e-09));
        assert_eq!(ephemeris.get_weeks(), Some(782));

        assert_eq!(
            ephemeris.get_orbit_f64("svAccuracy"),
            Some(0.200000000000e+01)
        );
        assert_eq!(ephemeris.get_orbit_f64("satH1"), Some(0.0));
        assert_eq!(
            ephemeris.get_orbit_f64("tgd1b1b3"),
            Some(-0.599999994133e-09)
        );
        assert_eq!(
            ephemeris.get_orbit_f64("tgd2b2b3"),
            Some(-0.900000000000e-08)
        );

        assert_eq!(ephemeris.get_orbit_f64("t_tm"), Some(0.432000000000e+06));
        assert_eq!(ephemeris.get_orbit_f64("aodc"), Some(0.0));
    }
    #[test]
    fn test_kepler2ecef() {
        let orbits = build_orbits(
            Constellation::GPS,
            vec![
                ("deltaN", "4.3123e-9"),
                ("gpsWeek", "910"),
                ("toe", "410400"),
                ("e", "4.27323824e-3"),
                ("m0", "2.24295542"),
                ("i0", "0.97477102"),
                ("idot", "-4.23946e-10"),
                ("sqrta", "5.15353571e3"),
                ("cuc", "-6.60121440e-6"),
                ("cus", "5.31412661e-6"),
                ("cic", "9.8720193e-8"),
                ("cis", "-3.9115548e-8"),
                ("crc", "282.28125"),
                ("crs", "-132.71875"),
                ("omega", "-0.88396725"),
                ("omega0", "2.29116688"),
                ("omegaDot", "-8.025691e-9"),
            ],
        );
        let ephemeris = Ephemeris {
            clock_bias: -0.426337239332e-03,
            clock_drift: -0.752518047875e-10,
            clock_drift_rate: 0.000000000000e+00,
            orbits,
        };

        let epoch = Epoch::from_time_of_week(910, 4.0327293e14 as u64, TimeScale::GPST);
        let xyz = ephemeris.kepler2ecef(epoch);

        assert!(xyz.is_some());
        let (x, y, z) = xyz.unwrap();
        assert!((x - -5678509.38584636).abs() < 1E-6);
        assert!((y - -24923975.356725316).abs() < 1E-6);
        assert!((z - 7056393.437932).abs() < 1E-6);

        let orbits = build_orbits(
            Constellation::GPS,
            vec![
                ("deltaN", "3.86730381052e-09"),
                ("gpsWeek", "2190.0"),
                ("toe", "432000.0"),
                ("e", "0.0112139617559"),
                ("m0", "-0.659513670614"),
                ("i0", "0.986440321199"),
                ("idot", "-2.98226721096e-10"),
                ("sqrta", "5153.67701149"),
                ("cuc", "-6.64032995701e-06"),
                ("cus", "7.05942511559e-06"),
                ("cic", "-9.31322574615e-09"),
                ("cis", "2.10478901863e-07"),
                ("crc", "255.375"),
                ("crs", "-126.90625"),
                ("omega", "0.883585650969"),
                ("omega0", "-1.03593017273"),
                ("omegaDot", "-7.99890464975e-09"),
            ],
        );
        let ephemeris = Ephemeris {
            clock_bias: -0.426337239332e-03,
            clock_drift: -0.752518047875e-10,
            clock_drift_rate: 0.000000000000e+00,
            orbits,
        };
        let epoch = Epoch::from_time_of_week(2190, 1324944000 * 1_000_000_000, TimeScale::GPST);
        let xyz = ephemeris.kepler2ecef(epoch);

        assert!(xyz.is_some());
        let (x, y, z) = xyz.unwrap();
        assert!((x - -11840614.01333711).abs() < 1E-6);
        assert!((y - 19224209.93574417).abs() < 1E-6);
        assert!((z - 13435836.30353981).abs() < 1E-6);
    }
}
