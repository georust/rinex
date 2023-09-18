use super::{orbits::closest_nav_standards, NavMsgType, OrbitItem};
use crate::{epoch, prelude::*, sv, version::Version};

use hifitime::GPST_REF_EPOCH;
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
    #[error("sv parsing error")]
    SvParsing(#[from] sv::ParsingError),
    #[error("failed to identify timescale for sv \"{0}\"")]
    TimescaleIdentification(Sv),
}

/// Ephermeris NAV frame type
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Ephemeris {
    /// Clock bias (in seconds)
    pub clock_bias: f64,
    /// Clock drift (s.s⁻¹)
    pub clock_drift: f64,
    /// Clock drift rate (s.s⁻²)).   
    pub clock_drift_rate: f64,
    /// Orbits are revision and constellation dependent,
    /// sorted by key and content, described in navigation::database
    pub orbits: HashMap<String, OrbitItem>,
}

/// Kepler parameters
#[cfg(feature = "nav")]
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
pub struct Kepler {
    /// semi major axis (m)
    pub a: f64,
    /// Eccentricity (n.a)
    pub e: f64,
    /// Inclination angle at reference time (semicircles)
    pub i_0: f64,
    /// Longitude of ascending node at reference time (semicircles)
    pub omega_0: f64,
    /// Mean anomaly at reference time (semicircles)
    pub m_0: f64,
    /// argument of perigee (semicircles)
    pub omega: f64,
    /// time of issue of ephemeris
    pub toe: f64,
}

#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
impl Kepler {
    /// Eearth mass * Gravitationnal field constant [m^3/s^2]
    pub const EARTH_GM_CONSTANT: f64 = 3.986004418E14_f64;
    /// Earth rotation rate in WGS84 frame [rad]
    pub const EARTH_OMEGA_E_WGS84: f64 = 7.2921151467E-5;
}

/// Orbit Perturbations
#[cfg(feature = "nav")]
#[derive(Default, Clone, Debug)]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// Retrieve all Sv clock biases (error, drift, drift rate).
    pub fn sv_clock(&self) -> (f64, f64, f64) {
        (self.clock_bias, self.clock_drift, self.clock_drift_rate)
    }
    /// Retrieves orbit data field expressed as f64 value, if such field exists.
    pub fn get_orbit_f64(&self, field: &str) -> Option<f64> {
        if let Some(v) = self.orbits.get(field) {
            v.as_f64()
        } else {
            None
        }
    }
    /*
     * Adds an orbit entry, mostly used when inserting
     * Kepler & Perturbations parameters in testing workflows.
     */
    pub(crate) fn set_orbit_f64(&mut self, field: &str, value: f64) {
        self.orbits
            .insert(field.to_string(), OrbitItem::from(value));
    }
    /*
     * Retrieves week counter, if such data exists
     */
    pub(crate) fn get_week(&self) -> Option<u32> {
        self.orbits.get("week").and_then(|field| field.as_u32())
    }
    /*
     * Retrieves toe, expressed as an Epoch, if Week + TOE are properly received
     */
    pub(crate) fn toe(&self, ts: TimeScale) -> Option<Epoch> {
        let week = self.get_week()?;
        let toe_f64 = self.get_orbit_f64("toe")?;
        Some(Epoch::from_time_of_week(
            week,
            toe_f64.round() as u64 * 1_000_000_000,
            ts,
        ))
    }
    /*
     * Parses ephemeris from given line iterator
     */
    pub(crate) fn parse_v2v3(
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
        let (clk_bias, rem) = rem.split_at(19);
        let (clk_dr, clk_drr) = rem.split_at(19);

        let mut sv = Sv::default();
        let mut epoch = Epoch::default();

        match version.major {
            1 | 2 => {
                match constellation {
                    Constellation::Mixed => {
                        // not sure that even exists
                        sv = Sv::from_str(svnn.trim())?
                    },
                    _ => {
                        sv.constellation = constellation;
                        sv.prn = u8::from_str_radix(svnn.trim(), 10)?;
                    },
                }
            },
            3 => {
                sv = Sv::from_str(svnn.trim())?;
            },
            _ => unreachable!("V4 is treated in a dedicated method"),
        };

        let ts = sv
            .constellation
            .to_timescale()
            .ok_or(Error::TimescaleIdentification(sv))?;
        //println!("V2/V3 CONTENT \"{}\" TIMESCALE {}", line, ts);

        let (epoch, _) = epoch::parse_in_timescale(date.trim(), sv.constellation)?;

        let clock_bias = f64::from_str(clk_bias.replace("D", "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace("D", "E").trim())?;
        let clock_drift_rate = f64::from_str(clk_drr.replace("D", "E").trim())?;
        // parse orbits :
        //  only Legacy Frames in V2 and V3 (old) RINEX
        let orbits = parse_orbits(version, NavMsgType::LNAV, sv.constellation, lines)?;
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
    /*
     * Parses ephemeris from given line iterator
     * RINEX V4 content specific method
     */
    pub(crate) fn parse_v4(
        msg: NavMsgType,
        mut lines: std::str::Lines<'_>,
    ) -> Result<(Epoch, Sv, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::MissingData),
        };

        let (svnn, rem) = line.split_at(4);
        let sv = Sv::from_str(svnn.trim())?;
        let (epoch, rem) = rem.split_at(19);

        let ts = sv
            .constellation
            .to_timescale()
            .ok_or(Error::TimescaleIdentification(sv))?;
        //println!("V4 CONTENT \"{}\" TIMESCALE {}", line, ts);

        let (epoch, _) = epoch::parse_in_timescale(epoch.trim(), sv.constellation)?;

        let (clk_bias, rem) = rem.split_at(19);
        let (clk_dr, clk_drr) = rem.split_at(19);
        let clock_bias = f64::from_str(clk_bias.replace("D", "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace("D", "E").trim())?;
        let clock_drift_rate = f64::from_str(clk_drr.replace("D", "E").trim())?;
        let orbits = parse_orbits(Version { major: 4, minor: 0 }, msg, sv.constellation, lines)?;
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

//#[cfg(feature = "nav")]
//use std::collections::BTreeMap;

#[cfg(feature = "nav")]
impl Ephemeris {
    /// Retrieves Orbit Keplerian parameters
    pub fn kepler(&self) -> Option<Kepler> {
        Some(Kepler {
            a: self.get_orbit_f64("sqrta")?.powf(2.0),
            e: self.get_orbit_f64("e")?,
            i_0: self.get_orbit_f64("i0")?,
            omega: self.get_orbit_f64("omega")?,
            omega_0: self.get_orbit_f64("omega0")?,
            m_0: self.get_orbit_f64("m0")?,
            toe: self.get_orbit_f64("toe")?,
        })
    }
    /// Creates new Ephemeris with given [`OrbitItem`]
    pub fn with_orbit(&self, key: &str, orbit: OrbitItem) -> Self {
        let mut s = self.clone();
        s.orbits.insert(key.to_string(), orbit);
        s
    }
    /// Creates new Ephemeris with given week counter
    pub fn with_week(&self, week: u32) -> Self {
        self.with_orbit("week", OrbitItem::from(week))
    }
    /// Creates new Ephemeris with given [`Kepler`] parameters
    pub fn with_kepler(&self, kepler: Kepler) -> Self {
        let mut s = self.clone();
        s.set_orbit_f64("sqrta", kepler.a.sqrt());
        s.set_orbit_f64("e", kepler.e);
        s.set_orbit_f64("i0", kepler.i_0);
        s.set_orbit_f64("omega", kepler.omega);
        s.set_orbit_f64("omega0", kepler.omega_0);
        s.set_orbit_f64("m0", kepler.m_0);
        s.set_orbit_f64("toe", kepler.toe);
        s
    }
    /// Retrieves Orbit [Perturbations] parameters
    pub fn perturbations(&self) -> Option<Perturbations> {
        Some(Perturbations {
            cuc: self.get_orbit_f64("cuc")?,
            cus: self.get_orbit_f64("cus")?,
            cic: self.get_orbit_f64("cic")?,
            cis: self.get_orbit_f64("cis")?,
            crc: self.get_orbit_f64("crc")?,
            crs: self.get_orbit_f64("crs")?,
            dn: self.get_orbit_f64("deltaN")?,
            i_dot: self.get_orbit_f64("idot")?,
            omega_dot: self.get_orbit_f64("omegaDot")?,
        })
    }
    /// Creates new Ephemeris with given Orbit [Perturbations]
    pub fn with_perturbations(&self, perturbations: Perturbations) -> Self {
        let mut s = self.clone();
        s.set_orbit_f64("cuc", perturbations.cuc);
        s.set_orbit_f64("cus", perturbations.cus);
        s.set_orbit_f64("cic", perturbations.cic);
        s.set_orbit_f64("cis", perturbations.cis);
        s.set_orbit_f64("crc", perturbations.crc);
        s.set_orbit_f64("crs", perturbations.crs);
        s.set_orbit_f64("deltaN", perturbations.dn);
        s.set_orbit_f64("idot", perturbations.i_dot);
        s.set_orbit_f64("omegaDot", perturbations.omega_dot);
        s
    }
    /*
    * Manual calculations of satellite position vector, in km ECEF.
    * `t_sv`: orbit epoch as parsed in RINEX.
    * TODO: this is currently only verified in GPST
            need to verify GST/BDT/IRNSST support
    * See [Bibliography::AsceAppendix3] and [Bibliography::JLe19]
    */
    pub(crate) fn kepler2ecef(&self, sv: &Sv, epoch: Epoch) -> Option<(f64, f64, f64)> {
        // To form t_sv : we need to convert UTC time to GNSS time.
        // Hifitime v4, once released, will help here
        let mut t_sv = epoch.clone();

        match sv.constellation {
            Constellation::GPS | Constellation::QZSS => {
                t_sv.time_scale = TimeScale::GPST;
                t_sv -= Duration::from_seconds(18.0); // GPST(t=0) number of leap seconds @ the time
            },
            Constellation::Galileo => {
                t_sv.time_scale = TimeScale::GST;
                t_sv -= Duration::from_seconds(31.0); // GST(t=0) number of leap seconds @ the time
            },
            Constellation::BeiDou => {
                t_sv.time_scale = TimeScale::BDT;
                t_sv -= Duration::from_seconds(32.0); // BDT(t=0) number of leap seconds @ the time
            },
            _ => {}, // either not needed, or most probably not truly supported
        }

        let kepler = self.kepler()?;
        let perturbations = self.perturbations()?;

        let weeks = self.get_week()?;
        let t0 = GPST_REF_EPOCH + Duration::from_days((weeks * 7).into());
        let toe = t0 + Duration::from_seconds(kepler.toe as f64);
        let t_k = (t_sv - toe).to_seconds();

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

        Some((x_k / 1000.0, y_k / 1000.0, z_k / 1000.0))
    }
    /*
     * Returns Sv position in km ECEF, based off Self Ephemeris data,
     * and for given Satellite Vehicle at given Epoch.
     * Either by solving Kepler equations, or directly if such data is available.
     */
    pub(crate) fn sv_position(&self, sv: &Sv, epoch: Epoch) -> Option<(f64, f64, f64)> {
        let (x_km, y_km, z_km) = (
            self.get_orbit_f64("satPosX"),
            self.get_orbit_f64("satPosY"),
            self.get_orbit_f64("satPosZ"),
        );
        match (x_km, y_km, z_km) {
            (Some(x_km), Some(y_km), Some(z_km)) => {
                /*
                 * GLONASS + SBAS: position vector already available,
                 *                 distances expressed in km ECEF
                 */
                Some((x_km, y_km, z_km))
            },
            _ => self.kepler2ecef(sv, epoch),
        }
    }
    /*
     * Computes elev, azim angles both in degrees
     */
    pub(crate) fn sv_elev_azim(
        &self,
        sv: &Sv,
        epoch: Epoch,
        reference: GroundPosition,
    ) -> Option<(f64, f64)> {
        let (sv_x, sv_y, sv_z) = self.sv_position(sv, epoch)?;
        let (ref_x, ref_y, ref_z) = reference.to_ecef_wgs84();
        // convert ref position to radians(lat, lon)
        let (ref_lat, ref_lon, _) =
            map_3d::ecef2geodetic(ref_x, ref_y, ref_z, map_3d::Ellipsoid::WGS84);

        // ||sv - ref_pos|| pseudo range
        let a_i = (
            sv_x * 1000.0 - ref_x,
            sv_y * 1000.0 - ref_y,
            sv_z * 1000.0 - ref_z,
        );
        let norm = (a_i.0.powf(2.0) + a_i.1.powf(2.0) + a_i.2.powf(2.0)).sqrt();
        let a_i = (a_i.0 / norm, a_i.1 / norm, a_i.2 / norm);

        // ECEF to VEN 3X3 transform matrix
        let ecef_to_ven = (
            (
                ref_lat.cos() * ref_lon.cos(),
                ref_lat.cos() * ref_lon.sin(),
                ref_lat.sin(),
            ),
            (-ref_lon.sin(), ref_lon.cos(), 0.0_f64),
            (
                -ref_lat.sin() * ref_lon.cos(),
                -ref_lat.sin() * ref_lon.sin(),
                ref_lat.cos(),
            ),
        );
        // ECEF to VEN transform
        let ven = (
            (ecef_to_ven.0 .0 * a_i.0 + ecef_to_ven.0 .1 * a_i.1 + ecef_to_ven.0 .2 * a_i.2),
            (ecef_to_ven.1 .0 * a_i.0 + ecef_to_ven.1 .1 * a_i.1 + ecef_to_ven.1 .2 * a_i.2),
            (ecef_to_ven.2 .0 * a_i.0 + ecef_to_ven.2 .1 * a_i.1 + ecef_to_ven.2 .2 * a_i.2),
        );
        let el = map_3d::rad2deg(std::f64::consts::PI / 2.0 - ven.0.acos());
        let mut az = map_3d::rad2deg(ven.1.atan2(ven.2));
        if az < 0.0 {
            az += 360.0;
        }
        Some((el, az))
    }
    /*
     * Returns max time difference between an Epoch and
     * related Time of Issue of Ephemeris, for each constellation.
     */
    pub(crate) fn max_dtoe(c: Constellation) -> Option<Duration> {
        match c {
            Constellation::GPS | Constellation::QZSS | Constellation::Geo => {
                Some(Duration::from_seconds(7200.0))
            },
            Constellation::Galileo => Some(Duration::from_seconds(10800.0)),
            Constellation::BeiDou => Some(Duration::from_seconds(21600.0)),
            Constellation::SBAS(_) => Some(Duration::from_seconds(360.0)),
            Constellation::IRNSS => Some(Duration::from_seconds(86400.0)),
            Constellation::Glonass => Some(Duration::from_seconds(1800.0)),
            _ => None,
        }
    }
}

/*
 * Parses constellation + revision dependent orbits data fields.
 * Retrieves all of this information from the databased stored and maintained
 * in db/NAV/orbits.
 */
fn parse_orbits(
    version: Version,
    msg: NavMsgType,
    constell: Constellation,
    lines: std::str::Lines<'_>,
) -> Result<HashMap<String, OrbitItem>, Error> {
    // Determine closest standards from DB
    // <=> data fields to parse
    let nav_standards = match closest_nav_standards(constell, version, msg) {
        Some(v) => v,
        _ => return Err(Error::DataBaseRevisionError),
    };

    //println!("FIELD : {:?} \n", nav_standards.items); // DEBUG

    let fields = &nav_standards.items;

    let mut key_index: usize = 0;
    let word_size: usize = 19;
    let mut map: HashMap<String, OrbitItem> = HashMap::new();

    for line in lines {
        // trim first few white spaces
        let mut line: &str = match version.major < 3 {
            true => &line[3..],
            false => &line[4..],
        };

        let mut nb_missing = 4 - (line.len() / word_size);
        //println!("LINE \"{}\" | NB MISSING {}", line, nb_missing); //DEBUG

        loop {
            if line.len() == 0 {
                key_index += nb_missing as usize;
                break;
            }

            let (content, rem) = line.split_at(std::cmp::min(word_size, line.len()));

            if content.trim().len() == 0 {
                // omitted field
                key_index += 1;
                if nb_missing > 0 {
                    nb_missing -= 1;
                }
                line = rem.clone();
                continue;
            }

            if let Some((key, token)) = fields.get(key_index) {
                //println!(
                //    "Key \"{}\"(index: {}) | Token \"{}\" | Content \"{}\"",
                //    key,
                //    key_index,
                //    token,
                //    content.trim()
                //); //DEBUG
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
#[cfg(feature = "nav")]
mod epoch_serde {
    use crate::prelude::Epoch;
    use serde::{self, Deserialize, Deserializer};
    use std::str::FromStr;
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Epoch, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            if let Ok(e) = Epoch::from_str(&s) {
                Ok(e)
            } else {
                panic!("failed to deserialize epoch");
            }
        } else {
            panic!("failed to deserialize epoch");
        }
    }
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
            if key.contains("Week") {
                map.insert(
                    key.to_string(),
                    OrbitItem::new("u32", value, constellation).unwrap(),
                );
            } else {
                map.insert(
                    key.to_string(),
                    OrbitItem::new("f64", value, constellation).unwrap(),
                );
            }
        }
        map
    }
    #[test]
    fn gal_orbit() {
        let content =
            "     7.500000000000e+01 1.478125000000e+01 2.945479833915e-09-3.955466341850e-01
     8.065253496170e-07 3.683507675305e-04-3.911554813385e-07 5.440603218079e+03
     3.522000000000e+05-6.519258022308e-08 2.295381450845e+00 7.450580596924e-09
     9.883726443393e-01 3.616875000000e+02 2.551413130998e-01-5.907746081337e-09
     1.839362331110e-10 2.580000000000e+02 2.111000000000e+03                   
     3.120000000000e+00 0.000000000000e+00-1.303851604462e-08 0.000000000000e+00
     3.555400000000e+05";
        let orbits = parse_orbits(
            Version::new(3, 0),
            NavMsgType::LNAV,
            Constellation::Galileo,
            content.lines(),
        );
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
        assert_eq!(ephemeris.get_week(), Some(2111));

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
        let content =
            "      .100000000000e+01  .118906250000e+02  .105325815814e-08 -.255139531119e+01
      .169500708580e-06  .401772442274e-03  .292365439236e-04  .649346986580e+04
      .432000000000e+06  .105705112219e-06 -.277512444499e+01 -.211410224438e-06
      .607169709798e-01 -.897671875000e+03  .154887266488e+00 -.871464871438e-10
     -.940753471872e-09  .000000000000e+00  .782000000000e+03  .000000000000e+00
      .200000000000e+01  .000000000000e+00 -.599999994133e-09 -.900000000000e-08
      .432000000000e+06  .000000000000e+00 0.000000000000e+00 0.000000000000e+00";
        let orbits = parse_orbits(
            Version::new(3, 0),
            NavMsgType::LNAV,
            Constellation::BeiDou,
            content.lines(),
        );
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
        assert_eq!(ephemeris.get_week(), Some(782));

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
    fn glonass_orbit_v2() {
        let content =
            "   -1.488799804690D+03-2.196182250980D+00 3.725290298460D-09 0.000000000000D+00
    1.292880712890D+04-2.049269676210D+00 0.000000000000D+00 1.000000000000D+00
    2.193169775390D+04 1.059645652770D+00-9.313225746150D-10 0.000000000000D+00";
        let orbits = parse_orbits(
            Version::new(2, 0),
            NavMsgType::LNAV,
            Constellation::Glonass,
            content.lines(),
        );
        assert!(orbits.is_ok(), "failed to parse Glonass V2 orbits");
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("satPosX"), Some(-1.488799804690E3));
        assert_eq!(ephemeris.get_orbit_f64("satPosY"), Some(1.292880712890E4));
        assert_eq!(ephemeris.get_orbit_f64("satPosZ"), Some(2.193169775390E4));
    }
    #[test]
    fn glonass_orbit_v3() {
        let content =
            "      .783916601562e+04 -.423131942749e+00  .931322574615e-09  .000000000000e+00
     -.216949155273e+05  .145034790039e+01  .279396772385e-08  .300000000000e+01
      .109021518555e+05  .319181251526e+01  .000000000000e+00  .000000000000e+00";
        let orbits = parse_orbits(
            Version::new(3, 0),
            NavMsgType::LNAV,
            Constellation::Glonass,
            content.lines(),
        );
        assert!(orbits.is_ok(), "failed to parse Glonass V3 orbits");
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("satPosX"), Some(0.783916601562E4));
        assert_eq!(ephemeris.get_orbit_f64("satPosY"), Some(-0.216949155273E5));
        assert_eq!(ephemeris.get_orbit_f64("satPosZ"), Some(0.109021518555E5));
    }
    #[test]
    fn glonass_orbit_v2_missing_fields() {
        let content =
            "   -1.488799804690D+03                    3.725290298460D-09 0.000000000000D+00
    1.292880712890D+04-2.049269676210D+00 0.000000000000D+00 1.000000000000D+00
    2.193169775390D+04 1.059645652770D+00-9.313225746150D-10 0.000000000000D+00";
        let orbits = parse_orbits(
            Version::new(2, 0),
            NavMsgType::LNAV,
            Constellation::Glonass,
            content.lines(),
        );
        assert!(orbits.is_ok(), "failed to parse Glonass V2 orbits");
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("satPosX"), Some(-1.488799804690E3));
        assert_eq!(ephemeris.get_orbit_f64("velX"), None);
        assert_eq!(ephemeris.get_orbit_f64("satPosY"), Some(1.292880712890E4));
        assert_eq!(ephemeris.get_orbit_f64("satPosZ"), Some(2.193169775390E4));
    }
    #[test]
    fn glonass_orbit_v3_missing_fields() {
        let content =
            "      .783916601562e+04                    .931322574615e-09  .000000000000e+00
     -.216949155273e+05  .145034790039e+01  .279396772385e-08  .300000000000e+01
      .109021518555e+05  .319181251526e+01  .000000000000e+00  .000000000000e+00";
        let orbits = parse_orbits(
            Version::new(3, 0),
            NavMsgType::LNAV,
            Constellation::Glonass,
            content.lines(),
        );
        assert!(orbits.is_ok(), "failed to parse Glonass V3 orbits");
        let orbits = orbits.unwrap();
        let ephemeris = Ephemeris {
            clock_bias: 0.0,
            clock_drift: 0.0,
            clock_drift_rate: 0.0,
            orbits,
        };
        assert_eq!(ephemeris.get_orbit_f64("satPosX"), Some(0.783916601562E4));
        assert_eq!(ephemeris.get_orbit_f64("velX"), None);
        assert_eq!(ephemeris.get_orbit_f64("satPosY"), Some(-0.216949155273E5));
        assert_eq!(ephemeris.get_orbit_f64("satPosZ"), Some(0.109021518555E5));
    }
    use super::{Ephemeris, Kepler, Perturbations};
    use serde::Deserialize;
    #[derive(Default, Debug, Clone, Deserialize)]
    struct Helper {
        #[serde(with = "epoch_serde")]
        epoch: Epoch,
        sv: Sv,
        azi: f64,
        elev: f64,
        week: u32,
        kepler: Kepler,
        ecef: (f64, f64, f64),
        ref_pos: GroundPosition,
        perturbations: Perturbations,
    }
    fn helper_to_ephemeris(hp: Helper) -> Ephemeris {
        Ephemeris::default()
            .with_kepler(hp.kepler)
            .with_perturbations(hp.perturbations)
            .with_week(hp.week)
    }
    #[test]
    fn kepler_gpst() {
        let descriptors: Vec<&str> = vec![
            r#"
{
  "epoch": "2020-12-31T23:59:44.000000000 UTC",
  "sv": {
    "prn": 7,
    "constellation": "GPS"
  },
  "week": 2138,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [605350.1978036277,-20286526.552827496,17200398.126797352],
  "elev": 15.00220128288493,
  "azi": 300.68660523817476,
  "kepler": {
    "a": 26559660.946231633,
    "e": 0.0143113207305,
    "i_0": 0.951953396771,
    "omega_0": 2.33342477886,
    "m_0": -1.67314469571,
    "omega": -2.35693190038,
    "toe": 431984.0
  },
  "perturbations": {
    "dn": 2.543973073573274e-17,
    "i_dot": -1.59292343205e-10,
    "omega_dot": -8.03426303264e-09,
    "cus": 5.50784170628e-06,
    "cuc": -8.475035429e-07,
    "cis": -8.00937414169e-08,
    "cic": 2.21654772758e-07,
    "crs": -15.09375,
    "crc": 262.65625
  }
}"#,
            r#"
{
  "epoch": "2021-01-02T00:00:00.000000000 UTC",
  "sv": {
    "prn": 18,
    "constellation": "GPS"
  },
  "week": 2138,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [257853.85371909104,19563995.622981288,17931314.572146047],
  "elev": 26.125733114760926,
  "azi": 68.34811299624617,
  "kepler": {
    "a": 26560589.686413657,
    "e": 0.00118253775872,
    "i_0": 0.966406105756,
    "omega_0": -0.795452281483,
    "m_0": -0.839958584081,
    "omega": 3.01997025999,
    "toe": 518400.0
  },
  "perturbations": {
    "dn": 2.06749838401754e-17,
    "i_dot": -2.05365696671e-10,
    "omega_dot": -8.41106473359e-09,
    "cus": 8.62404704094e-07,
    "cuc": -2.65054404736e-06,
    "cis": -7.63684511185e-08,
    "cic": -2.79396772385e-08,
    "crs": -49.96875,
    "crc": 367.125
  }
}"#,
            r#"
{
  "epoch": "2021-01-02T00:00:00.000000000 UTC",
  "sv": {
    "prn": 30,
    "constellation": "GPS"
  },
  "week": 2138,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [-8565700.261484932,-13909486.809253218,20957103.36075533],
  "elev": 11.018002970007121,
  "azi": 329.0430784548838,
  "kepler": {
    "a": 26561204.90386163,
    "e": 0.00474791659508,
    "i_0": 0.937190900254,
    "omega_0": 2.35208528936,
    "m_0": -1.64976237865,
    "omega": -2.84623407963,
    "toe": 518400.0
  },
  "perturbations": {
    "dn": 2.9993768567594165e-17,
    "i_dot": -7.00029159024e-11,
    "omega_dot": -8.43535136624e-09,
    "cus": 5.39235770702e-06,
    "cuc": -6.07222318649e-07,
    "cis": -2.421438694e-08,
    "cic": 7.63684511185e-08,
    "crs": -7.5,
    "crc": 261.46875
  }
}"#,
            r#"
{
  "epoch": "2021-12-31T22:00:00.000000000 UTC",
  "sv": {
    "prn": 8,
    "constellation": "GPS"
  },
  "week": 2190,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [7360759.045154838,-20964798.98238912,14276873.329646083],
  "elev": 18.884894173760276,
  "azi": 282.6253915920682,
  "kepler": {
    "a": 26560621.613883533,
    "e": 0.00704538438004,
    "i_0": 0.965195715133,
    "omega_0": -2.11600985989,
    "m_0": 0.637626493874,
    "omega": 0.0714426386841,
    "toe": 511200.0
  },
  "perturbations": {
    "dn": 2.214895343799872e-17,
    "i_dot": 1.78578867098e-11,
    "omega_dot": -8.54178437103e-09,
    "cus": -4.56348061562e-07,
    "cuc": 3.36021184921e-06,
    "cis": 5.58793544769e-08,
    "cic": 1.32247805595e-07,
    "crs": 63.25,
    "crc": 384.46875
  }
}"#,
            r#"
{
  "epoch": "2022-01-01T00:00:00.000000000 UTC",
  "sv": {
    "prn": 32,
    "constellation": "GPS"
  },
  "week": 2190,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [16685968.411769923,20728763.631397538,-1574846.006229475],
  "elev": 8.386332281745226,
  "azi": 133.44087594021298,
  "kepler": {
    "a": 26561110.712759566,
    "e": 0.00534839148168,
    "i_0": 0.957537602313,
    "omega_0": 1.03791041521,
    "m_0": 2.30316624652,
    "omega": -2.3834050415,
    "toe": 518400.0
  },
  "perturbations": {
    "dn": 2.3949035344821167e-17,
    "i_dot": 5.11807041192e-10,
    "omega_dot": -8.0467641439e-09,
    "cus": 6.09830021858e-06,
    "cuc": 9.85339283943e-07,
    "cis": -1.54599547386e-07,
    "cic": -1.04308128357e-07,
    "crs": 17.3125,
    "crc": 258.34375
  }
}"#,
            r#"
{
  "epoch": "2021-12-30T20:00:00.000000000 UTC",
  "sv": {
    "prn": 11,
    "constellation": "GPS"
  },
  "week": 2190,
  "ref_pos": [3628427.9118,562059.0936,5197872.215],
  "ecef": [-16564151.460786693,12177059.553538049,16806283.53619841],
  "elev": -2.06436127635181,
  "azi": 34.06512647418462,
  "kepler": {
    "a": 26561225.448593915,
    "e": 0.000327356392518,
    "i_0": 0.961559997976,
    "omega_0": -0.990953651689,
    "m_0": -0.519517248299,
    "omega": 2.77982583653,
    "toe": 417600.0
  },
  "perturbations": {
    "dn": 1.7909501455594142e-17,
    "i_dot": -1.72864347836e-10,
    "omega_dot": -7.98926169665e-09,
    "cus": 8.02055001259e-06,
    "cuc": -7.46361911297e-06,
    "cis": -9.49949026108e-08,
    "cic": 4.65661287308e-08,
    "crs": -144.84375,
    "crc": 229.125
  }
}"#,
        ];
        // test all descriptors
        for descriptor in descriptors {
            let helper = serde_json::from_str::<Helper>(descriptor);
            assert!(helper.is_ok(), "faulty test data description");
            let helper = helper.unwrap();

            // parse
            let ephemeris = helper_to_ephemeris(helper.clone());
            assert!(
                ephemeris.kepler().is_some(),
                "kepler parameters setup failed"
            );
            assert!(
                ephemeris.perturbations().is_some(),
                "orbit perturbations setup failed"
            );
            assert!(
                ephemeris.get_week().is_some(),
                "missing week counter, context is faulty"
            );

            // solver
            let ecef = ephemeris.sv_position(&helper.sv, helper.epoch);
            assert!(
                ecef.is_some(),
                "kepler2ecef should be feasible with provided context"
            );

            let ecef = ecef.unwrap();

            let x_err = (ecef.0 * 1000.0 - helper.ecef.0).abs();
            let y_err = (ecef.1 * 1000.0 - helper.ecef.1).abs();
            let z_err = (ecef.2 * 1000.0 - helper.ecef.2).abs();
            assert!(x_err < 1E-6, "kepler2ecef: x_err too large: {}", x_err);
            assert!(y_err < 1E-6, "kepler2ecef: y_err too large: {}", y_err);
            assert!(z_err < 1E-6, "kepler2ecef: z_err too large: {}", z_err);

            let el_az = ephemeris.sv_elev_azim(&helper.sv, helper.epoch, helper.ref_pos);
            assert!(
                el_az.is_some(),
                "sv_elev_azim: should have been feasible in this context!"
            );

            let (elev, azim) = el_az.unwrap();
            let el_err = (elev - helper.elev).abs();
            let az_err = (azim - helper.azi).abs();
            assert!(el_err < 1E-6, "sv_elev: error too large: {}", el_err);
            assert!(az_err < 1E-6, "sv_azim: error too large: {}", az_err);
        }
    }
}
