use super::{orbits::closest_nav_standards, NavMsgType, OrbitItem};
use crate::constants::Constants;
use crate::{
    constants, epoch,
    prelude::{Almanac, Constellation, Duration, Epoch, TimeScale, SV},
    version::Version,
};

use anise::{
    astro::AzElRange,
    constants::frames::{EARTH_J2000, IAU_EARTH_FRAME},
    errors::AlmanacResult,
    prelude::{Frame, Orbit},
};

use log::{error, warn};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

use crate::epoch::{
    parse_in_timescale as parse_epoch_in_timescale, ParsingError as EpochParsingError,
};

#[cfg(feature = "nav")]
use nalgebra::{self as na, Rotation, Rotation3, Vector3, Vector4};

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
    EpochParsing(#[from] EpochParsingError),
    #[error("sv parsing error")]
    SvParsing(#[from] gnss::sv::ParsingError),
    #[error("failed to identify timescale for sv \"{0}\"")]
    TimescaleIdentification(SV),
}

/// EphemerisHelper
/// # Action
/// - Helps calculate satellite orbits described by Keplerian orbital elements, inculding
/// GPS、BDS、Galieo
/// - Helps calculate relativistic effects(todo)
#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
#[derive(Debug, Clone, Copy)]
struct EphemerisHelper {
    /// Satellite
    pub sv: SV,
    /// The difference between the calculated time and the ephemeris reference time
    pub t_k: f64,
    /// Ascending angle(corrected)
    pub u_k: f64,
    /// Radius(corrected)
    pub r_k: f64,
    /// Orbital inclination(corrected)
    pub i_k: f64,
    /// Ascending node right ascension
    pub omega_k: f64,
    /// First Derivative of Ascending angle(corrected)
    pub fd_u_k: f64,
    /// First Derivative of Radius(corrected)
    pub fd_r_k: f64,
    /// First Derivative of Orbital inclination(corrected)
    pub fd_i_k: f64,
    /// First Derivative of Ascending node right ascension
    pub fd_omega_k: f64,
    /// orbit position
    pub orbit_position: (f64, f64),
    /// Relativistic Effect Correction
    pub dtr: f64,
    /// First Derivative of Relativistic Effect Correction
    pub fd_dtr: f64,
    /// Orbit
    orbit: Orbit,
}

#[cfg(feature = "nav")]
#[cfg_attr(docrs, doc(cfg(feature = "nav")))]
impl EphemerisHelper {
    fn meo_orbit_to_ecef_rotation_matrix(&self) -> Rotation<f64, 3> {
        // Positive angles mean counterclockwise rotation
        let rotation_x = Rotation3::from_axis_angle(&Vector3::x_axis(), self.i_k);
        let rotation_z = Rotation3::from_axis_angle(&Vector3::z_axis(), self.omega_k);
        rotation_z * rotation_x
    }

    fn geo_orbit_to_ecef_rotation_matrix(&self) -> Rotation<f64, 3> {
        let rotation_x = Rotation::from_axis_angle(&Vector3::x_axis(), 5.0f64.to_radians());
        let rotation_z =
            Rotation::from_axis_angle(&Vector3::z_axis(), -constants::Omega::BDS * self.t_k);
        rotation_z * rotation_x
    }

    fn orbit_velocity(&self) -> (f64, f64) {
        let (sin_u_k, cos_u_k) = self.u_k.sin_cos();
        let fd_x = self.fd_r_k * cos_u_k - self.r_k * self.fd_u_k * sin_u_k;
        let fd_y = self.fd_r_k * sin_u_k + self.r_k * self.fd_u_k * cos_u_k;
        (fd_x, fd_y)
    }

    /// Calculate ecef position [km].
    /// Applies to GPS, Galileo, BeiDou (MEO).
    fn ecef_position(&self) -> Vector3<f64> {
        let orbit_xyz = Vector3::new(self.orbit_position.0, self.orbit_position.1, 0.0);
        let ecef_xyz = self.meo_orbit_to_ecef_rotation_matrix() * orbit_xyz;
        ecef_xyz / 1000.0
    }

    /// Calculate ecef velocity [km/s].
    /// Applies to GPS, Galileo, BeiDou (MEO).
    fn ecef_velocity(&self) -> Vector3<f64> {
        let (x, y) = self.orbit_position;
        let (sin_omega_k, cos_omega_k) = self.omega_k.sin_cos();
        let (sin_i_k, cos_i_k) = self.i_k.sin_cos();
        // First Derivative of orbit position
        let (fd_x, fd_y) = self.orbit_velocity();
        // First Derivative of rotation Matrix
        let mut fd_r = na::SMatrix::<f64, 3, 4>::zeros();
        fd_r[(0, 0)] = cos_omega_k;
        fd_r[(0, 1)] = -sin_omega_k * cos_i_k;
        fd_r[(0, 2)] = -(x * sin_omega_k + y * cos_omega_k * cos_i_k);
        fd_r[(0, 3)] = y * sin_omega_k * sin_i_k;
        fd_r[(1, 0)] = sin_omega_k;
        fd_r[(1, 1)] = cos_omega_k * cos_i_k;
        fd_r[(1, 2)] = x * cos_omega_k - y * sin_omega_k * cos_i_k;
        fd_r[(1, 3)] = y * cos_omega_k * sin_i_k;
        fd_r[(2, 1)] = sin_i_k;
        fd_r[(2, 3)] = y * cos_i_k;

        let rhs = Vector4::new(fd_x, fd_y, self.fd_omega_k, self.fd_i_k);
        let vel = fd_r * rhs;
        vel / 1000.0
    }

    /// Calculate ECEF position [km] and velocity [km/s] of MEO/IGSO sv
    /// # Return
    /// ( Position(x,y,z),Velecity(x,y,z) )
    fn ecef_pv(&self) -> (Vector3<f64>, Vector3<f64>) {
        (self.ecef_position(), self.ecef_velocity())
    }

    /// Calculate ecef [km] position of GEO sv
    fn beidou_geo_ecef_position(&self) -> Vector3<f64> {
        let orbit_xyz = Vector3::new(self.orbit_position.0, self.orbit_position.1, 0.0);
        let rotation1 = self.meo_orbit_to_ecef_rotation_matrix();
        let rotation2 = self.geo_orbit_to_ecef_rotation_matrix();
        let ecef_xyz = rotation2 * rotation1 * orbit_xyz;
        ecef_xyz / 1000.0
    }

    /// Calculate ecef velocity of GEO sv
    fn beidou_geo_ecef_velocity(&self) -> Vector3<f64> {
        let (x, y) = self.orbit_position;
        let (sin_omega_k, cos_omega_k) = self.omega_k.sin_cos();
        let (sin_i_k, cos_i_k) = self.i_k.sin_cos();
        let (fd_x, fd_y) = self.orbit_velocity();
        let fd_xgk = -y * self.fd_omega_k - fd_y * cos_i_k * sin_omega_k + fd_x * cos_omega_k;
        let fd_ygk = x * self.fd_omega_k + fd_y * cos_i_k * cos_omega_k + fd_x * sin_omega_k;
        let fd_zgk = fd_y * sin_i_k + y * self.fd_i_k * cos_i_k;

        let rx = Rotation3::from_axis_angle(&Vector3::x_axis(), 5.0);
        let rz = Rotation3::from_axis_angle(&Vector3::z_axis(), -constants::Omega::BDS * self.t_k);
        let (sin_omega_tk, cos_omega_tk) = (constants::Omega::BDS * self.t_k).sin_cos();
        let fd_rz = self.fd_omega_k
            * na::Matrix3::new(
                -sin_omega_tk,
                cos_omega_tk,
                0.0,
                -cos_omega_tk,
                -sin_omega_tk,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        let pos = self.beidou_geo_ecef_position();
        let fd_pos = Vector3::new(fd_xgk, fd_ygk, fd_zgk);
        let vel = fd_rz * rx * pos + rz * rx * fd_pos;
        vel
    }

    /// Calculate ecef position and velocity of BeiDou GEO sv
    /// # Return
    /// ( Position(x,y,z),Velecity(x,y,z) )
    fn beidou_geo_ecef_pv(&self) -> (Vector3<f64>, Vector3<f64>) {
        let (x, y) = self.orbit_position;
        let (sin_omega_k, cos_omega_k) = self.omega_k.sin_cos();
        let (sin_i_k, cos_i_k) = self.i_k.sin_cos();
        let (fd_x, fd_y) = self.orbit_velocity();
        let fd_xgk = -y * self.fd_omega_k - fd_y * cos_i_k * sin_omega_k + fd_x * cos_omega_k;
        let fd_ygk = x * self.fd_omega_k + fd_y * cos_i_k * cos_omega_k + fd_x * sin_omega_k;
        let fd_zgk = fd_y * sin_i_k + y * self.fd_i_k * cos_i_k;

        let rx = Rotation3::from_axis_angle(&Vector3::x_axis(), 5.0);
        let rz = Rotation3::from_axis_angle(&Vector3::z_axis(), -constants::Omega::BDS * self.t_k);
        let (sin_omega_tk, cos_omega_tk) = (constants::Omega::BDS * self.t_k).sin_cos();
        let fd_rz = self.fd_omega_k
            * na::Matrix3::new(
                -sin_omega_tk,
                cos_omega_tk,
                0.0,
                -cos_omega_tk,
                -sin_omega_tk,
                0.0,
                0.0,
                0.0,
                0.0,
            );
        let pos = self.beidou_geo_ecef_position();
        let fd_pos = Vector3::new(fd_xgk, fd_ygk, fd_zgk);
        let vel = fd_rz * rx * pos + rz * rx * fd_pos;
        (pos, vel)
    }

    /// get ecef position
    pub fn position(&self) -> Option<Vector3<f64>> {
        match self.sv.constellation {
            Constellation::GPS | Constellation::Galileo => Some(self.ecef_position()),
            Constellation::BeiDou => {
                if self.sv.is_beidou_geo() {
                    Some(self.beidou_geo_ecef_position())
                } else {
                    Some(self.ecef_position())
                }
            },
            _ => {
                warn!("{} is not supported", self.sv.constellation);
                None
            },
        }
    }

    /// get ecef position and velocity
    pub fn position_velocity(&self) -> Option<(Vector3<f64>, Vector3<f64>)> {
        if self.sv.is_beidou_geo() {
            Some(self.beidou_geo_ecef_pv())
        } else {
            match self.sv.constellation {
                Constellation::GPS | Constellation::Galileo | Constellation::BeiDou => {
                    Some(self.ecef_pv())
                },
                _ => {
                    warn!("{} is not supported", self.sv.constellation);
                    None
                },
            }
        }
    }
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
    /// Retrieve all SV clock biases (error, drift, drift rate).
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

    /// Adds an orbit entry field, encoding a double precision number.
    pub(crate) fn set_orbit_f64(&mut self, field: &str, value: f64) {
        self.orbits
            .insert(field.to_string(), OrbitItem::from(value));
    }
    /// Try to retrive the week counter. This exists
    /// for all Constellations expect [Constellation::Glonass].
    pub(crate) fn get_week(&self) -> Option<u32> {
        self.orbits.get("week").and_then(|value| value.as_u32())
    }
    /*
     * Returns TGD field, if such field is not empty, expressed as a [Duration]
     */
    pub fn tgd(&self) -> Option<Duration> {
        Some(Duration::from_seconds(self.get_orbit_f64("tgd")?))
    }
    /// Return ToE expressed as [Epoch]
    pub fn toe(&self, sv_ts: TimeScale) -> Option<Epoch> {
        // TODO: in CNAV V4 TOC is said to be TOE... ...
        let mut week = self.get_week()?;
        let sec = self.get_orbit_f64("toe")?;
        let week_dur = Duration::from_days((week * 7) as f64);
        let sec_dur = Duration::from_seconds(sec);
        match sv_ts {
            TimeScale::GPST | TimeScale::QZSST | TimeScale::GST => {
                if sv_ts == TimeScale::GST {
                    week -= 1024;
                }
                Some(Epoch::from_duration(week_dur + sec_dur, TimeScale::GPST))
            },
            TimeScale::BDT => Some(Epoch::from_bdt_duration(week_dur + sec_dur)),
            _ => {
                error!("{} is not supported", sv_ts);
                None
            },
        }
    }
    /*
     * get Adot field in CNAV ephemeris
     */
    pub(crate) fn a_dot(&self) -> Option<f64> {
        self.get_orbit_f64("a_dot")
    }
    /// Parse Ephemeris (V2/V3) from line iterator
    pub(crate) fn parse_v2v3(
        version: Version,
        constellation: Constellation,
        mut lines: std::str::Lines<'_>,
    ) -> Result<(Epoch, SV, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::MissingData),
        };

        let svnn_offset: usize = match version.major < 3 {
            true => 3,
            false => 4,
        };

        let (svnn, rem) = line.split_at(svnn_offset);
        let (date, rem) = rem.split_at(19);
        let (clk_bias, rem) = rem.split_at(19);
        let (clk_dr, clk_drr) = rem.split_at(19);

        //log::debug!("SVNN \"{}\"", svnn);
        let sv = match SV::from_str(svnn.trim()) {
            Ok(sv) => sv,
            Err(_) => {
                // parsing failed probably due to omitted constellation (old rev...)
                let desc = format!("{:x}{:02}", constellation, svnn.trim());
                SV::from_str(&desc)?
            },
        };
        //log::debug!("\"{}\"={}", svnn, sv);

        let ts = sv
            .constellation
            .timescale()
            .ok_or(Error::TimescaleIdentification(sv))?;

        //log::debug!("V2/V3 CONTENT \"{}\" TIMESCALE {}", line, ts);

        let epoch = parse_epoch_in_timescale(date.trim(), ts)?;

        let clock_bias = f64::from_str(clk_bias.replace('D', "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace('D', "E").trim())?;
        let mut clock_drift_rate = f64::from_str(clk_drr.replace('D', "E").trim())?;

        // parse orbits :
        //  only Legacy Frames in V2 and V3 (old) RINEX
        let mut orbits = parse_orbits(version, NavMsgType::LNAV, sv.constellation, lines)?;

        if sv.constellation.is_sbas() {
            // SBAS frames specificity:
            // clock drift rate does not exist and is actually the week counter
            orbits.insert(
                "week".to_string(),
                OrbitItem::U32(clock_drift_rate.round() as u32),
            );
            clock_drift_rate = 0.0_f64; // drift rate null: non existing
        }

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
    /// Parse Ephemeris (V4) from line iterator
    pub(crate) fn parse_v4(
        msg: NavMsgType,
        mut lines: std::str::Lines<'_>,
        ts: TimeScale,
    ) -> Result<(Epoch, SV, Self), Error> {
        let line = match lines.next() {
            Some(l) => l,
            _ => return Err(Error::MissingData),
        };

        let (svnn, rem) = line.split_at(4);
        let sv = SV::from_str(svnn.trim())?;
        let (epoch, rem) = rem.split_at(19);
        let epoch = epoch::parse_in_timescale(epoch.trim(), ts)?;

        let (clk_bias, rem) = rem.split_at(19);
        let (clk_dr, clk_drr) = rem.split_at(19);
        let clock_bias = f64::from_str(clk_bias.replace('D', "E").trim())?;
        let clock_drift = f64::from_str(clk_dr.replace('D', "E").trim())?;
        let mut clock_drift_rate = f64::from_str(clk_drr.replace('D', "E").trim())?;
        let mut orbits =
            parse_orbits(Version { major: 4, minor: 0 }, msg, sv.constellation, lines)?;

        if sv.constellation.is_sbas() {
            // SBAS frames specificity:
            // clock drift rate does not exist and is actually the week counter
            orbits.insert(
                "week".to_string(),
                OrbitItem::U32(clock_drift_rate.round() as u32),
            );
            clock_drift_rate = 0.0_f64; // drift rate null: non existing
        }

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
    /// Total seconds elapsed between `t` and ToE, expressed in GPS timescale.
    fn t_k_gpst_s(&self, sv: SV, t: Epoch) -> Option<f64> {
        let sv_ts = sv.timescale()?;
        let toe_gpst = self.toe(sv_ts)?.to_time_scale(TimeScale::GPST);
        let dt = t.to_time_scale(TimeScale::GPST) - toe_gpst;
        Some(dt.to_seconds())
    }

    /// Form ephemerisHelper
    fn ephemeris_helper(&self, sv: SV, t: Epoch) -> Option<EphemerisHelper> {
        // const
        let gm_m3_s2 = Constants::gm(sv);
        let omega = Constants::omega(sv);
        let dtr_f = Constants::dtr_f(sv);
        let t_k = self.t_k_gpst_s(sv, t)?;

        if t_k < 0.0 {
            error!("t_k < 0.0: bad op");
            return None;
        }

        let mut kepler = self.kepler()?;
        let perturbations = self.perturbations()?;

        // considering the filed a_dot
        if let Some(a_dot) = self.a_dot() {
            kepler.a += a_dot * t_k;
        }

        let n0 = (gm_m3_s2 / kepler.a.powi(3)).sqrt(); // average angular velocity
        let n = n0 + perturbations.dn; // corrected mean angular velocity
        let m_k = kepler.m_0 + n * t_k; // average anomaly

        // Iterative calculation of e_k
        let mut e_k_lst: f64 = 0.0;
        let mut e_k: f64 = 0.0;
        let mut i = 0;
        loop {
            e_k = m_k + kepler.e * e_k_lst.sin();
            if (e_k - e_k_lst).abs() < 1e-10 {
                break;
            }
            i += 1;
            e_k_lst = e_k;
        }
        if i >= constants::MaxIterNumber::KEPLER {
            error!("{} kepler iteration overflow", sv);
        }

        // true anomaly
        let (sin_e_k, cos_e_k) = e_k.sin_cos();
        let v_k = ((1.0 - kepler.e.powi(2)).sqrt() * sin_e_k).atan2(cos_e_k - kepler.e);

        let phi_k = v_k + kepler.omega; // latitude argument
        let (x2_sin_phi_k, x2_cos_phi_k) = (2.0 * phi_k).sin_cos();

        // latitude argument correction
        let du_k = perturbations.cus * x2_sin_phi_k + perturbations.cuc * x2_cos_phi_k;
        let u_k = phi_k + du_k;

        // orbital radisu correction
        let dr_k = perturbations.crs * x2_sin_phi_k + perturbations.crc * x2_cos_phi_k;
        let r_k = kepler.a * (1.0 - kepler.e * e_k.cos()) + dr_k;

        // inclination angle correction
        let di_k = perturbations.cis * x2_sin_phi_k + perturbations.cic * x2_cos_phi_k;

        // first derivatives
        let fd_omega_k = perturbations.omega_dot - omega;

        let fd_e_k = n / (1.0 - kepler.e * e_k.cos());
        let fd_phi_k = ((1.0 + kepler.e) / (1.0 - kepler.e)).sqrt()
            * ((v_k / 2.0).cos() / (e_k / 2.0).cos()).powi(2)
            * fd_e_k;

        let fd_u_k =
            (perturbations.cus * x2_cos_phi_k - perturbations.cuc * x2_sin_phi_k) * fd_phi_k * 2.0
                + fd_phi_k;

        let fd_r_k = kepler.a * kepler.e * e_k.sin() * fd_e_k
            + 2.0
                * (perturbations.crs * x2_cos_phi_k - perturbations.crc * x2_sin_phi_k)
                * fd_phi_k;

        let fd_i_k = perturbations.i_dot
            + 2.0
                * (perturbations.cis * x2_cos_phi_k - perturbations.cic * x2_sin_phi_k)
                * fd_phi_k;

        // relativistic effect correction
        let dtr = dtr_f * kepler.e * kepler.a.sqrt() * e_k.sin();
        let fd_dtr = dtr_f * kepler.e * kepler.a.sqrt() * e_k.cos() * fd_e_k;

        // ascending node longitude correction (RAAN ?)
        let omega_k = if sv.is_beidou_geo() {
            // IGSO(BeiDou)
            kepler.omega_0 + perturbations.omega_dot * t_k - omega * kepler.toe
        } else {
            // MEO (GPS, Galileo, BeiDou)
            kepler.omega_0 + (perturbations.omega_dot - omega) * t_k - omega * kepler.toe
        };

        // corrected inclination angle
        let i_k = kepler.i_0 + di_k + perturbations.i_dot * t_k;

        // position in orbital plane
        let orbit_position = (r_k * u_k.cos(), r_k * u_k.sin());

        // Finally, determine Orbital state
        let orbit = Orbit::try_keplerian(
            kepler.a * 1e-3,
            kepler.e,
            i_k.to_degrees(),
            omega_k.to_degrees(),
            omega.to_degrees(),
            v_k.to_degrees(),
            t.to_time_scale(TimeScale::GPST),
            EARTH_J2000.with_mu_km3_s2(gm_m3_s2 * 1e-9),
        )
        .ok()?;

        Some(EphemerisHelper {
            sv,
            t_k,
            orbit,
            omega_k,
            dtr,
            fd_dtr,
            u_k,
            i_k,
            fd_u_k,
            r_k,
            fd_r_k,
            fd_i_k,
            fd_omega_k,
            orbit_position,
        })
    }
    /// Kepler ECEF [km] position solver at desired instant "t" for given "sv"
    /// based off Self. Self must be correctly selected in navigation
    /// record.
    /// See [Bibliography::AsceAppendix3], [Bibliography::JLe19] and [Bibliography::BeiDouICD]
    pub fn kepler2position(&self, sv: SV, t: Epoch) -> Option<(f64, f64, f64)> {
        let helper = self.ephemeris_helper(sv, t)?;
        let pos = helper.ecef_position();
        Some((pos.x, pos.y, pos.z))
    }
    /// Kepler ECEF [km] position and velocity [km/s] solver at desired instant "t" for given "sv"
    /// based off Self. Self must be correctly selected in navigation
    /// record.
    /// "t" should not be expressed in UTC time scale as the hifitime doesn't consider
    /// the leap seconds
    /// See [Bibliography::AsceAppendix3], [Bibliography::JLe19] and [Bibliography::BeiDouICD]
    pub fn kepler2position_velocity(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Option<((f64, f64, f64), (f64, f64, f64))> {
        let helper = self.ephemeris_helper(sv, t)?;
        let (pos, vel) = helper.position_velocity()?;
        Some((
            (pos.x / 1000.0, pos.y / 1000.0, pos.z / 1000.0),
            (vel.x, vel.y, vel.z),
        ))
    }
    /// [AzElRange] calculation attempt, for following SV as observed at RX,
    /// both coordinates expressed as [km] in ECEF frame.
    pub fn elevation_azimuth_range(
        t: Epoch,
        almanac: &Almanac,
        fixed_body_frame: Frame,
        sv_position: (f64, f64, f64),
        rx_position: (f64, f64, f64),
    ) -> AlmanacResult<AzElRange> {
        let (rx_x_km, rx_y_km, rx_z_km) = (
            rx_position.0 / 1000.0,
            rx_position.1 / 1000.0,
            rx_position.2 / 1000.0,
        );
        let (tx_x_km, tx_y_km, tx_z_km) = (
            sv_position.0 / 1000.0,
            sv_position.1 / 1000.0,
            sv_position.2 / 1000.0,
        );
        almanac.azimuth_elevation_range_sez(
            Orbit::from_position(tx_x_km, tx_y_km, tx_z_km, t, fixed_body_frame),
            Orbit::from_position(rx_x_km, rx_y_km, rx_z_km, t, fixed_body_frame),
        )
    }
    /// Returns True if Self is Valid at specified `t`
    pub fn is_valid(&self, sv: SV, t: Epoch) -> bool {
        if let Some(max_dt) = Self::max_dtoe(sv.constellation) {
            if let Some(sv_ts) = sv.constellation.timescale() {
                if let Some(toe) = self.toe(sv_ts) {
                    t > toe && (t - toe) <= max_dt
                } else {
                    error!("{}({}): failed to determine ToE", t, sv);
                    false
                }
            } else {
                error!("{} constellation is not supported", sv.constellation);
                false
            }
        } else {
            error!("{} constellation is not supported", sv.constellation);
            false
        }
    }
    /// Returns Ephemeris validity duration for this Constellation
    pub fn max_dtoe(c: Constellation) -> Option<Duration> {
        match c {
            Constellation::GPS | Constellation::QZSS => Some(Duration::from_seconds(7200.0)),
            Constellation::Galileo => Some(Duration::from_seconds(10800.0)),
            Constellation::BeiDou => Some(Duration::from_seconds(21600.0)),
            Constellation::IRNSS => Some(Duration::from_seconds(7200.0)),
            Constellation::Glonass => Some(Duration::from_seconds(1800.0)),
            c => {
                if c.is_sbas() {
                    // tolerate one publication per day
                    Some(Duration::from_seconds(86.4E3))
                } else {
                    None
                }
            },
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
    // convert SBAS constell to compatible "sbas" (undetermined/general constell)
    let constell = match constell.is_sbas() {
        true => Constellation::SBAS,
        false => constell,
    };
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
            if line.is_empty() {
                key_index += nb_missing;
                break;
            }

            let (content, rem) = line.split_at(std::cmp::min(word_size, line.len()));
            let content = content.trim();

            if content.is_empty() {
                // omitted field
                key_index += 1;
                nb_missing = nb_missing.saturating_sub(1);
                line = rem;
                continue;
            }
            /*
             * In NAV RINEX, unresolved data fields are either
             * omitted (handled previously) or put a zeros
             */
            if !content.contains(".000000000000E+00") {
                if let Some((key, token)) = fields.get(key_index) {
                    //println!(
                    //    "Key \"{}\"(index: {}) | Token \"{}\" | Content \"{}\"",
                    //    key,
                    //    key_index,
                    //    token,
                    //    content.trim()
                    //); //DEBUG
                    if !key.contains("spare") {
                        if let Ok(item) = OrbitItem::new(token, content, constell) {
                            map.insert(key.to_string(), item);
                        }
                    }
                }
            }
            key_index += 1;
            line = rem;
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
            if key.contains("week") {
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
}
