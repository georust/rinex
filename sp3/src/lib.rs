//! SP3 precise orbit file parser.
#![doc(html_logo_url = "https://raw.githubusercontent.com/georust/meta/master/logo/logo.png")]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;

use itertools::Itertools;

#[cfg(feature = "qc")]
extern crate rinex_qc_traits as qc_traits;

use gnss::prelude::{Constellation, SV};
use hifitime::{Epoch, ParsingError as EpochParsingError};

use std::{collections::BTreeMap, io::Error as IoError};

use gnss_rs::constellation::ParsingError as ConstellationParsingError;
use thiserror::Error;

#[cfg(feature = "qc")]
#[cfg_attr(docsrs, doc(cfg(feature = "qc")))]
mod qc;

#[cfg(feature = "processing")]
#[cfg_attr(docsrs, doc(cfg(feature = "processing")))]
mod processing;

#[cfg(feature = "anise")]
use anise::{
    astro::AzElRange,
    math::Vector6,
    prelude::{Almanac, Frame, Orbit},
};

#[cfg(test)]
mod tests;

mod header;
mod parsing;
mod position;
mod velocity;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use header::Header;
use hifitime::Unit;

type Vector3D = (f64, f64, f64);

pub mod prelude {
    pub use crate::{
        header::{version::Version, DataType, Header, OrbitType},
        Error, ParsingError, SP3Entry, SP3Key, SP3,
    };
    // Pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale};

    #[cfg(feature = "qc")]
    pub use rinex_qc_traits::{Merge, Split};
}

/// [SP3Entry] indexer
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3Key {
    /// Spacecraft described as [SV]
    pub sv: SV,
    /// Epoch
    pub epoch: Epoch,
}

/// [SP3Entry] record file content, sorted per [SP3Key]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3Entry {
    /// ECEF position in kilometers with 10⁻³ precision.
    pub position_km: Vector3D,
    /// ECEF velocity vectori in km.s⁻¹.
    pub velocity_km_s: Option<Vector3D>,
    /// True if the state vector is predicted
    pub orbit_prediction: bool,
    /// True if vehicle being maneuvered (rocket truster)
    /// since last state.
    pub maneuver: bool,
    /// Discontinuity in the satellite clock correction
    /// (for example: internal clock swap)
    pub clock_event: bool,
    /// True when the clock state is actually predicted
    pub clock_prediction: bool,
    /// Clock offset correction, in microsecond with 10⁻¹² precision.
    pub clock_us: Option<f64>,
    /// Clock drift in nanoseconds with 10⁻¹⁶ precision.
    pub clock_drift_ns: Option<f64>,
}

impl SP3Entry {
    /// Builds new [SP3Entry] with "true" position and all other
    /// fields are unknown.
    pub fn from_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            velocity_km_s: None,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: false,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with position prediction, in kilometers.
    pub fn from_predicted_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            velocity_km_s: None,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: true,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with "true" position and velocity vector,
    /// any other fields are unknown.
    pub fn from_position_velocity_km_km_s(position_km: Vector3D, velocity_km_s: Vector3D) -> Self {
        Self {
            position_km,
            velocity_km_s: Some(velocity_km_s),
            clock_us: None,
            maneuver: false,
            clock_drift_ns: None,
            clock_prediction: false,
            orbit_prediction: false,
            clock_event: false,
        }
    }

    /// Builds new [SP3Entry] with predicted position and velocity vectors,
    /// all other fields are unknown.
    pub fn from_predicted_position_velocity_km_km_s(
        position_km: Vector3D,
        velocity_km_s: Vector3D,
    ) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            clock_drift_ns: None,
            velocity_km_s: Some(velocity_km_s),
            clock_prediction: false,
            orbit_prediction: true,
            clock_event: false,
        }
    }

    /// Copies and returns [SP3Entry] with "true" position vector.
    pub fn with_position_km(&self, position_km: Vector3D) -> Self {
        let mut s = self.clone();
        s.position_km = position_km;
        s.orbit_prediction = false;
        s
    }

    /// Copies and returns [SP3Entry] with predicted position vector.
    pub fn with_predicted_position_km(&self, position_km: Vector3D) -> Self {
        let mut s = self.clone();
        s.position_km = position_km;
        s.orbit_prediction = true;
        s
    }

    /// Copies and returns [SP3Entry] with "true" velocity vector
    pub fn with_velocity_km_s(&self, velocity_km_s: Vector3D) -> Self {
        let mut s = self.clone();
        s.velocity_km_s = Some(velocity_km_s);
        s.orbit_prediction = false;
        s
    }

    /// Copies and returns [SP3Entry] with predicted velocity vector
    pub fn with_predicted_velocity_km_s(&self, velocity_km_s: Vector3D) -> Self {
        let mut s = self.clone();
        s.velocity_km_s = Some(velocity_km_s);
        s.orbit_prediction = true;
        s
    }

    /// Copies and returns [Self] with "true" clock offset in seconds
    pub fn with_clock_offset_s(&self, offset_s: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_s * 1.0E6);
        s.clock_prediction = false;
        s
    }

    /// Copies and returns [Self] with predicted clock offset in seconds
    pub fn with_predicted_clock_offset_s(&self, offset_s: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_s * 1.0E6);
        s.clock_prediction = true;
        s
    }

    /// Copies and returns [Self] with "true" clock offset in microseconds
    pub fn with_clock_offset_us(&self, offset_us: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_us);
        s.clock_prediction = false;
        s
    }

    /// Copies and returns [Self] with predicted clock offset in microseconds
    pub fn with_predicted_clock_offset_us(&self, offset_us: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_us);
        s.clock_prediction = true;
        s
    }

    /// Copies and returns [Self] with clock drift in seconds
    pub fn with_clock_drift_s(&self, drift_s: f64) -> Self {
        let mut s = self.clone();
        s.clock_drift_ns = Some(drift_s * 1.0E9);
        s
    }

    /// Copies and returns [Self] with clock drift in nanoseconds
    pub fn with_clock_drift_ns(&self, drift_ns: f64) -> Self {
        let mut s = self.clone();
        s.clock_drift_ns = Some(drift_ns);
        s
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3 {
    /// File [Header]
    pub header: Header,
    /// File header comments, stored as is.
    pub comments: Vec<String>,
    /// File content are [SP3Entry]s sorted per [SP3Key]
    pub data: BTreeMap<SP3Key, SP3Entry>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Parsing error: {0}")]
    ParsingError(#[from] ParsingError),
    #[error("Epoch parsing error: {0}")]
    HifitimeParsingError(#[from] EpochParsingError),
    #[error("Constellation parsing error: {0}")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("File i/o error: {0}")]
    FileIo(#[from] IoError),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Non supported SP3 revision")]
    NonSupportedRevision,
    #[error("Unknown SP3 orbit type")]
    UnknownOrbitType,
    #[error("Unknown SP3 data type")]
    UnknownDataType,
    #[error("malformed header line #1")]
    MalformedH1,
    #[error("malformed header line #2")]
    MalformedH2,
    #[error("malformed %c line \"{0}\"")]
    MalformedDescriptor(String),
    #[error("failed to parse epoch year from \"{0}\"")]
    EpochYear(String),
    #[error("failed to parse epoch month from \"{0}\"")]
    EpochMonth(String),
    #[error("failed to parse epoch day from \"{0}\"")]
    EpochDay(String),
    #[error("failed to parse epoch hours from \"{0}\"")]
    EpochHours(String),
    #[error("failed to parse epoch minutes from \"{0}\"")]
    EpochMinutes(String),
    #[error("failed to parse epoch seconds from \"{0}\"")]
    EpochSeconds(String),
    #[error("failed to parse epoch milliseconds from \"{0}\"")]
    EpochMilliSeconds(String),
    #[error("failed to parse number of epochs \"{0}\"")]
    NumberEpoch(String),
    #[error("failed to parse week counter")]
    WeekCounter(String),
    #[error("failed to parse hifitime::Epoch")]
    Epoch,
    #[error("failed to parse sample rate from \"{0}\"")]
    EpochInterval(String),
    #[error("failed to parse mjd start \"{0}\"")]
    Mjd(String),
    #[error("failed to parse sv from \"{0}\"")]
    SV(String),
    #[error("failed to parse (x, y, or z) coordinates from \"{0}\"")]
    Coordinates(String),
    #[error("failed to parse clock data from \"{0}\"")]
    Clock(String),
}

use crate::prelude::DataType;

// Lagrangian interpolator
pub(crate) fn lagrange_interpolation(
    order: usize,
    t: Epoch,
    x: Vec<(Epoch, Vector3D)>,
) -> Option<Vector3D> {
    let x_len = x.len();
    let mut polynomials = Vector3D::default();

    if x_len < order + 1 {
        return None;
    }

    for i in 0..order + 1 {
        let mut l_i = 1.0_f64;
        let (t_i, (x_km_i, y_km_i, z_km_i)) = x[i];

        for j in 0..order + 1 {
            let (t_j, _) = x[j];
            if j != i {
                l_i *= (t - t_j).to_seconds();
                l_i /= (t_i - t_j).to_seconds();
            }
        }

        polynomials.0 += x_km_i * l_i;
        polynomials.1 += y_km_i * l_i;
        polynomials.2 += z_km_i * l_i;
    }

    Some(polynomials)
}

// // 2D Linear interpolation
// pub(crate) fn linear_interpolation(
//     order: usize,
//     t: Epoch,
//     x: Vec<(Epoch, f64)>,
// ) -> Option<f64> {
//
//     let x_len = x.len();
//     if x_len < 2 {
//         return None;
//     }
//
//     let (x_0, y_0) = x[0];
//     let (x_1, y_1) = x[1];
//     let dt = (x_1 - x_0).to_seconds();
//     let mut dy = (x_1 - t).to_seconds() /dt * y_0;
//     dy += (t - x_0).to_seconds() /dt * y_1;
//     Some(dy)
// }

impl SP3 {
    /// Returns [Epoch] of first entry
    pub fn first_epoch(&self) -> Epoch {
        let mut t0_utc = Epoch::from_mjd_utc(self.header.mjd);

        if self.header.timescale.is_gnss() {
            t0_utc = Epoch::from_duration(
                t0_utc - self.header.timescale.reference_epoch(),
                self.header.timescale,
            );
        }

        t0_utc
    }

    /// Returns last [Epoch] to be found in this record.
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().last()
    }

    /// Returns true if this [SP3] has satellites velocity vector
    pub fn has_satellite_velocity(&self) -> bool {
        self.header.data_type == DataType::Velocity
    }

    /// Returns true if at least one state vector (whatever the constellation)
    /// was predicted
    pub fn has_satellite_positions_prediction(&self) -> bool {
        self.data
            .iter()
            .filter_map(|(k, v)| {
                if v.orbit_prediction {
                    Some((k, v))
                } else {
                    None
                }
            })
            .count()
            > 0
    }

    /// Returns true if this [SP3] has satellites clock offset
    pub fn has_satellite_clock_offset(&self) -> bool {
        self.satellites_clock_offset_sec_iter().count() > 0
    }

    /// Returns true if this [SP3] has satellites clock drift
    pub fn has_satellite_clock_drift(&self) -> bool {
        self.satellites_clock_drift_sec_sec_iter().count() > 0
    }

    /// Returns true if at least 1 [SV] (whatever the constellation) is being maneuvered
    /// during this entire time frame
    pub fn has_satellite_maneuver(&self) -> bool {
        self.satellites_epoch_maneuver_iter().count() > 0
    }

    /// Returns true if this [SP3] publication is correct, that is:
    /// - all data points are correctly evenly spaced in time
    /// according to the sampling interval.
    /// You should use this verification method prior any interpolation (post processing).
    pub fn has_steady_sampling(&self) -> bool {
        let dt = self.header.epoch_interval;
        let mut t = Epoch::default();
        let mut past_t = Option::<Epoch>::None;

        for now in self.epochs_iter() {
            if now > t {
                // new epoch
                if let Some(past_t) = past_t {
                    if now - past_t != dt {
                        return false;
                    }
                }
                t = now;
            }
            past_t = Some(now);
        }
        true
    }

    /// Returns total number of [Epoch] to be found
    pub fn total_epochs(&self) -> usize {
        self.epochs_iter().count()
    }

    /// Returns [Epoch] [Iterator]
    pub fn epochs_iter(&self) -> impl Iterator<Item = Epoch> + '_ {
        self.data.iter().map(|(k, _)| k.epoch).unique()
    }

    /// Returns a unique [Constellation] iterator
    pub fn constellations_iter(&self) -> impl Iterator<Item = Constellation> + '_ {
        self.satellites_iter().map(|sv| sv.constellation).unique()
    }

    /// File comments [Iterator].
    pub fn comments_iter(&self) -> impl Iterator<Item = &String> + '_ {
        self.comments.iter()
    }

    /// Returns a unique [SV] iterator
    pub fn satellites_iter(&self) -> impl Iterator<Item = SV> + '_ {
        self.header.satellites.iter().copied()
    }

    /// [SV] position attitude [Iterator], in kilometers ECEF, with theoretical 10⁻³m precision.  
    /// All coordinates expressed in Coordinates system (always fixed body frame).  
    /// NB: all satellites being maneuvered are sorted out, which makes this method
    /// compatible with navigation.
    pub fn satellites_position_km_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, Vector3D)> + '_> {
        Box::new(self.data.iter().filter_map(|(k, v)| {
            if !v.maneuver {
                Some((k.epoch, k.sv, v.position_km))
            } else {
                None
            }
        }))
    }

    /// [SV] [Orbit]al state [Iterator] with theoretical 10⁻³m precision.
    /// For this to be correct:
    /// - [Frame] must be ECEF
    /// - [Frame] should match the coordinates system described in [Header]
    /// NB: all satellites being maneuvered are sorted out, which makes this method
    /// compatible with navigation.
    #[cfg(feature = "anise")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anise")))]
    pub fn satellites_orbit_iter(
        &self,
        frame_cef: Frame,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, Orbit)> + '_> {
        Box::new(self.data.iter().filter_map(move |(k, v)| {
            if !v.maneuver {
                let (x_km, y_km, z_km) = v.position_km;
                let (vx_km_s, vy_km_s, vz_km_s) = match v.velocity_km_s {
                    Some((vx_km_s, vy_km_s, vz_km_s)) => (vx_km_s, vy_km_s, vz_km_s),
                    None => (0.0, 0.0, 0.0),
                };

                let pos_vel = Vector6::new(x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s);
                let orbit = Orbit::from_cartesian_pos_vel(pos_vel, k.epoch, frame_cef);
                Some((k.epoch, k.sv, orbit))
            } else {
                None
            }
        }))
    }

    /// [SV] (elevation, azimuth, range) attitude vector [Iterator], as [AzElRange].
    #[cfg(feature = "anise")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anise")))]
    pub fn satellites_elevation_azimuth_iter(
        &self,
        almanac: Almanac,
        frame_cef: Frame,
        rx_orbit: Orbit,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, AzElRange)> + '_> {
        Box::new(
            self.satellites_orbit_iter(frame_cef)
                .filter_map(move |(sv, t, tx_orbit)| {
                    if let Ok(elazrng) =
                        almanac.azimuth_elevation_range_sez(rx_orbit, tx_orbit, None, None)
                    {
                        Some((sv, t, elazrng))
                    } else {
                        None
                    }
                }),
        )
    }

    /// Returns ([Epoch], [SV]) [Iterator] where satellite maneuver is being reported
    pub fn satellites_epoch_maneuver_iter(&self) -> Box<dyn Iterator<Item = (Epoch, SV)> + '_> {
        Box::new(self.data.iter().filter_map(|(k, v)| {
            if v.maneuver {
                Some((k.epoch, k.sv))
            } else {
                None
            }
        }))
    }

    /// Returns ([Epoch], [SV]) [Iterator] where satellite clock
    /// event flag was asserted.
    pub fn satellites_epoch_clock_event_iter(&self) -> Box<dyn Iterator<Item = (Epoch, SV)> + '_> {
        Box::new(self.data.iter().filter_map(|(k, v)| {
            if v.clock_event {
                Some((k.epoch, k.sv))
            } else {
                None
            }
        }))
    }

    /// Returns an [Iterator] over [SV] velocity vector, in km.s⁻¹
    /// and 0.1 10⁻⁷m precision, for all satellites in correct Orbit (not being maneuvered).
    pub fn satellites_velocity_km_s_iter(
        &self,
    ) -> Box<dyn Iterator<Item = (Epoch, SV, Vector3D)> + '_> {
        Box::new(self.data.iter().filter_map(|(k, v)| {
            if !v.maneuver {
                let (vx_dm_s, vy_dm_s, vz_dm_s) = v.velocity_km_s?;
                Some((k.epoch, k.sv, (vx_dm_s, vy_dm_s, vz_dm_s)))
            } else {
                None
            }
        }))
    }

    /// [SV] clock offset in seconds (with 10⁻¹² theoretical precision) [Iterator].
    pub fn satellites_clock_offset_sec_iter(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let clock = v.clock_us? * 1.0E-6;
            Some((k.epoch, k.sv, clock))
        })
    }

    /// [SV] clock offset in s.s⁻¹ (with 10⁻¹⁶ theoretical precision) [Iterator].
    pub fn satellites_clock_drift_sec_sec_iter(
        &self,
    ) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let rate = v.clock_drift_ns? * 1.0E-9;
            Some((k.epoch, k.sv, rate))
        })
    }

    /// Designs an evenly spaced (in time) grouping of (x_km, y_km, z_km) coordinates
    /// for you to apply your own interpolation method (as a function pointer).
    /// NB:
    /// - This only works on correct SP3 publications with steady sample rate.
    /// - There is no internal verification here, you should verify the correctness
    ///  of the SP3 publication with [Self::has_steady_sampling] prior running this.
    /// ## Input
    /// - sv: selected [SV]
    /// - t: Interpolation [Epoch]
    /// - order: Interpolation order. Only odd interpolation order is supported.
    /// This method will panic on even interpolation order.
    /// - interp: function pointer for this order and epoch, and the time frame.
    /// The time frame being [(N +1)/2 * τ;  (N +1)/2 * τ].
    /// A 7th order will create an 8 data point window.
    /// ## Output
    /// - Your function pointer should return Option<(f64, f64, f64)>,
    /// and is summoned with `order`, `t` and the time frame.
    /// - This method will return None if `t` is either
    /// to early or too late with respect to interpolation order.
    /// That means, we only generate perfectly centered time frames,
    /// to minimize interpolation error.
    pub fn satellite_position_interpolate(
        &self,
        sv: SV,
        t: Epoch,
        order: usize,
        interp: fn(usize, Epoch, Vec<(Epoch, Vector3D)>) -> Option<Vector3D>,
    ) -> Option<Vector3D> {
        let odd_order = order % 2 > 0;
        if !odd_order {
            panic!("even interpolation order is not supported");
        }

        // delta interval for which we consider Epoch equality
        let smallest_dt = 2.0 * Unit::Nanosecond;

        let target_len = order + 1;
        let target_len_2 = target_len / 2;
        let target_len_2_1 = target_len_2 - 1;

        let mut past_t = Epoch::default();

        let mut t_x = Option::<Epoch>::None;
        let mut tx_perfect_match = false;
        let (mut w0_len, mut w1_len) = (0, 0);

        let mut window = Vec::<(Epoch, Vector3D)>::with_capacity(target_len);

        for (index_i, (t_i, sv_i, (x_i, y_i, z_i))) in
            self.satellites_position_km_iter().enumerate()
        {
            if sv_i != sv {
                past_t = t_i;
                continue;
            }

            // always push while maintaining correct size
            window.push((t_i, (x_i, y_i, z_i)));

            let win_len = window.len();
            if win_len > target_len {
                window.remove(0);
            }

            if t_x.is_none() {
                if past_t < t && t_i >= t {
                    // found t_x
                    w0_len = index_i;
                    t_x = Some(t_i);

                    if (t_i - t).abs() < smallest_dt {
                        tx_perfect_match = true;
                    }
                }
            } else {
                // stop when window has been gathered
                if index_i == w0_len + target_len_2 - 1 {
                    w1_len = target_len_2;
                    break;
                }
            }

            past_t = t_i;
        }

        if t_x.is_none() {
            return None;
        }

        // central point must not be too early
        if w0_len < target_len_2 {
            return None;
        }

        // println!("t_x={} [{} ; {}]", t_x, w0_len, w1_len); // DEBUG

        // window must be correctly centered on central point
        if tx_perfect_match {
            if w1_len < target_len_2_1 {
                return None;
            }
        } else {
            if w1_len < target_len_2 {
                return None;
            }
        }

        interp(order, t, window)
    }

    /// Applies the Lagrangian interpolation method
    /// at desired [Epoch] `t` using desired interpoation order,
    /// as per <https://www.math.univ-paris13.fr/~japhet/L2/2020-2021/Interpolation.pdf>
    /// NB:
    /// - this will panic on even interpolation orders
    /// - this will not interpolate (returns None) if [Epoch]
    /// is either too early or too late with respect to
    /// interpolation order.
    pub fn satellite_position_lagrangian_interpolation(
        &self,
        sv: SV,
        t: Epoch,
        order: usize,
    ) -> Option<Vector3D> {
        self.satellite_position_interpolate(sv, t, order, lagrange_interpolation)
    }

    /// Applies 9th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    /// See [Self::satellite_position_lagrangian_interpolation].
    pub fn satellite_position_lagrangian_9_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Option<Vector3D> {
        self.satellite_position_lagrangian_interpolation(sv, t, 9)
    }

    /// Applies 11th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    /// See [Self::satellite_position_lagrangian_interpolation].
    pub fn satellite_position_lagrangian_11_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Option<Vector3D> {
        self.satellite_position_lagrangian_interpolation(sv, t, 11)
    }

    /// Applies 17th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    /// See [Self::satellite_position_lagrangian_interpolation].
    pub fn satellite_position_lagrangian_17_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Option<Vector3D> {
        self.satellite_position_lagrangian_interpolation(sv, t, 17)
    }
}
