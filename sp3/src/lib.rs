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
use std::collections::BTreeMap;

use gnss_rs::constellation::ParsingError as ConstellationParsingError;
use thiserror::Error;

#[cfg(feature = "qc")]
mod qc;

#[cfg(feature = "processing")]
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

#[cfg(docsrs)]
mod bibliography;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use header::Header;

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
    /// Clock offset correction, in microsecond with 10⁻¹² precision.
    pub clock_us: Option<f64>,
    /// Clock drift in nanoseconds with 10⁻¹⁶ precision.
    pub clock_drift_ns: Option<f64>,
    /// ECEF position in kilometers with 10⁻³ precision.
    pub position_km: Vector3D,
    /// ECEF velocity vectori in km.s⁻¹.
    pub velocity_km_s: Option<Vector3D>,
    /// Vehicle being maneuvered since last Epoch
    pub maneuver: bool,
}

impl SP3Entry {
    /// Builds new [SP3Entry] with given position and all other
    /// fields are unknown.
    pub fn from_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            velocity_km_s: None,
            clock_drift_ns: None,
        }
    }

    /// Builds new [SP3Entry] with given position and velocity vector,
    /// any other fields are unknown.
    pub fn from_position_velocity_km_km_s(position_km: Vector3D, velocity_km_s: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            maneuver: false,
            clock_drift_ns: None,
            velocity_km_s: Some(velocity_km_s),
        }
    }

    /// Copies and returns [SP3Entry] with given position vector
    pub fn with_position_km(&self, position_km: Vector3D) -> Self {
        let mut s = self.clone();
        s.position_km = position_km;
        s
    }

    /// Copies and returns [SP3Entry] with given velocity vector
    pub fn with_velocity_km_s(&self, velocity_km_s: Vector3D) -> Self {
        let mut s = self.clone();
        s.velocity_km_s = Some(velocity_km_s);
        s
    }

    /// Copies and returns [Self] with given clock offset in seconds
    pub fn with_clock_offset_s(&self, offset_s: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_s * 1.0E6);
        s
    }

    /// Copies and returns [Self] with given clock offset in microseconds
    pub fn with_clock_offset_us(&self, offset_us: f64) -> Self {
        let mut s = self.clone();
        s.clock_us = Some(offset_us);
        s
    }

    /// Copies and returns [Self] with given clock drift in seconds
    pub fn with_clock_drift_s(&self, drift_s: f64) -> Self {
        let mut s = self.clone();
        s.clock_drift_ns = Some(drift_s * 1.0E9);
        s
    }

    /// Copies and returns [Self] with given clock drift in nanoseconds
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
    /// File content are [SP3Entry]s sorted per [SP3Key]
    pub data: BTreeMap<SP3Key, SP3Entry>,
    /// File header comments, stored as is.
    pub comments: Vec<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("hifitime parsing error")]
    HifitimeParsingError(#[from] EpochParsingError),
    #[error("constellation parsing error")]
    ConstellationParsing(#[from] ConstellationParsingError),
    #[error("unknown or non supported revision \"{0}\"")]
    UnknownVersion(String),
    #[error("unknown data type \"{0}\"")]
    UnknownDataType(String),
    #[error("unknown orbit type \"{0}\"")]
    UnknownOrbitType(String),
    #[error("file i/o error")]
    DataParsingError(#[from] std::io::Error),
    #[error("even interpolation order not supported")]
    EvenInterpolationOrder,
    #[error("unable to design interpolation window")]
    InterpolationWindow,
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("unknown or non supported revision \"{0}\"")]
    UnknownVersion(String),
    #[error("unknown data type \"{0}\"")]
    UnknownDataType(String),
    #[error("unknown orbit type \"{0}\"")]
    UnknownOrbitType(String),
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
) -> Vector3D {
    let mut polynomials = Vector3D::default();

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

    polynomials
}

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

    /// Returns last [Epoch] to be find in this record
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epochs_iter().last()
    }

    /// Returns true if this [SP3] has satellites velocity vector
    pub fn has_satellite_velocity(&self) -> bool {
        self.header.data_type == DataType::Velocity
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

    /// Returns true if this [SP3] publication is correct
    /// - all data points are correctly evenly spaced in time
    /// according to the sampling interval.
    /// You should use this verification method prior any interpolation (post processing).
    pub fn has_correct_sampling(&self) -> bool {
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

    /// [SV] position attitude in kilometers ECEF with theoretical 10⁻³m precision.
    /// NB: all satellites being maneuvered are sorted out, which makes this method
    /// compatible with navigation.
    /// All coordinates expressed in Coordinates system (always fixed body frame).
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

    /// [SV] position attitude [Iterator] expressed as [Orbit] in desired reference [Frame].
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

    // /// Interpolate Clock (offset) at desired "t" expressed in the timescale you want.
    // /// SP3 files usually have a 15' sampling interval which makes this operation
    // /// most likely incorrect. You should either use higher sample rate to reduce
    // /// the error generated by interpolation, or use different products like
    // /// high precision Clock RINEX files.
    // pub fn sv_clock_interpolate(&self, t: Epoch, sv: SV) -> Option<f64> {
    //     let before = self
    //         .sv_clock()
    //         .filter_map(|(clk_t, clk_sv, value)| {
    //             if clk_t <= t && clk_sv == sv {
    //                 Some((clk_t, value))
    //             } else {
    //                 None
    //             }
    //         })
    //         .last()?;
    //     let after = self
    //         .sv_clock()
    //         .filter_map(|(clk_t, clk_sv, value)| {
    //             if clk_t > t && clk_sv == sv {
    //                 Some((clk_t, value))
    //             } else {
    //                 None
    //             }
    //         })
    //         .reduce(|k, _| k)?;
    //     let (before_t, before_clk) = before;
    //     let (after_t, after_clk) = after;
    //     let dt = (after_t - before_t).to_seconds();
    //     let mut bias = (after_t - t).to_seconds() / dt * before_clk;
    //     bias += (t - before_t).to_seconds() / dt * after_clk;
    //     Some(bias)
    // }

    /// Designs an evenly spaced (in time) grouping of (x_km, y_km, z_km) attitude
    /// vectors for you to apply your own interpolation method (as a function pointer).
    /// NB:
    /// - This only works on correct evenly spaced (in time) SP3 publications.
    /// - There is no internal verification here, you should verify the correctness
    ///  of the SP3 publication with [Self::has_correct_sampling] prior running this.
    /// - Only odd interpolation order is currently supported (for simplicity),
    /// this returns [Error::EvenInterpolationOrder].
    /// For example, order 7 will design an 7 data point window.
    /// - If `t` is either too early or too late, with respect to interpolation order,
    /// we return [Error::InterpolationWindow] that you should catch.
    /// - The interpolation window is [(N +1)/2 * τ;  (N +1)/2 * τ],
    /// where N is the interpolation order and τ the sampling interval.
    ///
    /// TODO
    /// See [Bibliography::Japhet2021].
    /// We propose none (interpolation not feasible) if `t` the interpolation [Epoch] is too
    /// early or too late, with respect to interpolation order.
    /// In order to preserve SP3 precision, an interpolation order between 7 and 11 is recommended.
    pub fn satellite_position_interpolate(
        &self,
        sv: SV,
        t: Epoch,
        order: usize,
        interp: fn(usize, Epoch, Vec<(Epoch, Vector3D)>) -> Vector3D,
    ) -> Result<Vector3D, Error> {
        let odd_order = order % 2 > 0;
        if !odd_order {
            return Err(Error::EvenInterpolationOrder);
        }

        // determine central point (in time)
        let center = self
            .satellites_position_km_iter()
            .find(|(e, svnn, _)| *svnn == sv && (*e - t).abs() < self.header.epoch_interval);

        if center.is_none() {
            return Err(Error::InterpolationWindow);
        }

        let (center_t, _, _) = center.unwrap();

        // determine (first, last) epoch of the time window
        let dt = ((order - 1) / 2) as f64;
        let dt = dt * self.header.epoch_interval;

        // design time window
        let window = self
            .satellites_position_km_iter()
            .filter_map(|(e, svnn, v3d)| {
                if svnn == sv {
                    if e >= center_t - dt && e <= center_t + dt {
                        Some((e, v3d))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if window.len() != order {
            return Err(Error::InterpolationWindow);
        }

        Ok(interp(order, t, window))
    }

    /// Applies the Lagrangian interpolation method
    /// at desired [Epoch] `t` using desired interpoation order.
    /// NB:
    /// - only odd interpolation is supported, otherwise returns [Error::EvenInterpolationOrder]
    /// - returns [Error::InterpolationWindow] error if [Epoch] is too early or too late,
    /// with respect of interpolation order.
    pub fn satellite_position_lagrangian_interpolation(
        &self,
        sv: SV,
        t: Epoch,
        order: usize,
    ) -> Result<Vector3D, Error> {
        self.satellite_position_interpolate(sv, t, order, lagrange_interpolation)
    }

    /// Applies 9th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    pub fn satellite_position_lagrangian_9_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Result<Vector3D, Error> {
        self.satellite_position_lagrangian_interpolation(sv, t, 9)
    }

    /// Applies 11th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    pub fn satellite_position_lagrangian_11_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Result<Vector3D, Error> {
        self.satellite_position_lagrangian_interpolation(sv, t, 11)
    }

    /// Applies 17th order Lagrangian interpolation method, which is compatible with high precision geodesy.
    pub fn satellite_position_lagrangian_17_interpolation(
        &self,
        sv: SV,
        t: Epoch,
    ) -> Result<Vector3D, Error> {
        self.satellite_position_lagrangian_interpolation(sv, t, 17)
    }
}
