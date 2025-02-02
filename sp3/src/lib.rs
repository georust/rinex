//! SP3 precise orbit file parser.
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

/*
 * 3D position
 */
type Vector3D = (f64, f64, f64);

pub mod prelude {
    pub use crate::{
        header::{version::Version, DataType, Header, OrbitType},
        Error, ParsingError, SP3Entry, SP3Key, SP3,
    };
    // Pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale};
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
}

impl SP3Entry {
    /// Builds new [SP3Entry] with given position and all other
    /// fields are unknown.
    pub fn from_position_km(position_km: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
            velocity_km_s: None,
            clock_drift_ns: None,
        }
    }

    /// Builds new [SP3Entry] with given position and velocity vector,
    /// any other fields are unknown
    pub fn from_position_velocity_km_km_s(position_km: Vector3D, velocity_km_s: Vector3D) -> Self {
        Self {
            position_km,
            clock_us: None,
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

impl SP3 {
    /// Returns [Epoch] of first entry
    pub fn first_epoch(&self) -> Epoch {
        let mut t0_utc = Epoch::from_mjd_utc(self.header.mjd);

        if self.header.time_scale.is_gnss() {
            t0_utc = Epoch::from_duration(
                t0_utc - self.header.time_scale.reference_epoch(),
                self.header.time_scale,
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
        self.satellites_clock_offset_s_iter().count() > 0
    }

    /// Returns true if this [SP3] has satellites clock drift
    pub fn has_satellite_clock_drift(&self) -> bool {
        self.satellites_clock_drift_s_s_iter().count() > 0
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
    /// All coordinates expressed in Coordinates system (always fixed body frame).
    pub fn satellites_position_km_iter(&self) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.data
            .iter()
            .map(|(k, v)| (k.epoch, k.sv, v.position_km))
    }

    /// [SV] position attitude [Iterator] expressed as [Orbit] in desired reference [Frame].
    /// For this to be correct:
    /// - [Frame] must be ECEF
    /// - [Frame] should match the coordinates system described in [Header]
    #[cfg(feature = "anise")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anise")))]
    pub fn satellites_orbit_iter(
        &self,
        frame_cef: Frame,
    ) -> impl Iterator<Item = (Epoch, SV, Orbit)> + '_ {
        self.data.iter().map(move |(k, v)| {
            let (x_km, y_km, z_km) = v.position_km;
            let (vx_km_s, vy_km_s, vz_km_s) = match v.velocity_km_s {
                Some((vx_km_s, vy_km_s, vz_km_s)) => (vx_km_s, vy_km_s, vz_km_s),
                None => (0.0, 0.0, 0.0),
            };

            let pos_vel = Vector6::new(x_km, y_km, z_km, vx_km_s, vy_km_s, vz_km_s);
            let orbit = Orbit::from_cartesian_pos_vel(pos_vel, k.epoch, frame_cef);
            (k.epoch, k.sv, orbit)
        })
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

    /// Returns an [Iterator] over [SV] velocity vector, in km.s⁻¹
    /// and 0.1 10⁻⁷m precision.
    pub fn satellites_velocity_km_s_iter(
        &self,
    ) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let (vx_dm_s, vy_dm_s, vz_dm_s) = v.velocity_km_s?;
            Some((k.epoch, k.sv, (vx_dm_s, vy_dm_s, vz_dm_s)))
        })
    }

    /// [SV] clock offset in seconds (with 10⁻¹² theoreetical precision) [Iterator].
    pub fn satellites_clock_offset_s_iter(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let clock = v.clock_us? * 1.0E-6;
            Some((k.epoch, k.sv, clock))
        })
    }

    /// [SV] clock offset in s.s⁻¹ (with 10⁻¹⁶ theoretical precision) [Iterator].
    pub fn satellites_clock_drift_s_s_iter(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
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

    // /// Applies desired interpolation method to handpicked satellite attitudes
    // /// (evenly spaced in time).
    // /// ](N +1)/2 * τ;  T - (N +1)/2 * τ],
    // /// where N is the interpolation order, τ the epoch interval,
    // /// and T the last Epoch in this file. See [Bibliography::Japhet2021].
    // /// In order to preserve SP3 precision, an interpolation order between 7 and 11 is recommended.
    // /// NB: only odd interpolation order is currently supported.
    // pub fn satellites_position_interpolate(&self, sv: SV, t: Epoch, interp: Fn(), order: usize) -> Option<Vector3D> {
    //     let odd_order = order % 2 > 0;
    //     let sv_position: Vec<_> = self
    //         .satellites_position_iter()
    //         .filter_map(|(e, svnn, (x, y, z))| {
    //             if sv == svnn {
    //                 Some((e, (x, y, z)))
    //             } else {
    //                 None
    //             }
    //         })
    //         .collect();

    //     /*
    //      * Determine closest Epoch in time
    //      */
    //     let center = match sv_position
    //         .iter()
    //         .find(|(e, _)| (*e - t).abs() < self.epoch_interval)
    //     {
    //         Some(center) => center,
    //         None => {
    //             /*
    //              * Failed to determine central Epoch for this SV:
    //              * empty data set: should not happen
    //              */
    //             return None;
    //         },
    //     };

    //     // println!("CENTRAL EPOCH : {:?}", center); //DEBUG

    //     let center_pos = match sv_position.iter().position(|(e, _)| *e == center.0) {
    //         Some(center) => center,
    //         None => {
    //             /* will never happen at this point*/
    //             return None;
    //         },
    //     };

    //     let (min_before, min_after): (usize, usize) = match odd_order {
    //         true => ((order + 1) / 2, (order + 1) / 2),
    //         false => (order / 2, order / 2 + 1),
    //     };

    //     if center_pos < min_before || sv_position.len() - center_pos < min_after {
    //         /* can't design time window */
    //         return None;
    //     }

    //     let mut polynomials = Vector3D::default();
    //     let offset = center_pos - min_before;

    //     for i in 0..order + 1 {
    //         let mut li = 1.0_f64;
    //         let (e_i, (x_i, y_i, z_i)) = sv_position[offset + i];
    //         for j in 0..order + 1 {
    //             let (e_j, _) = sv_position[offset + j];
    //             if j != i {
    //                 li *= (t - e_j).to_seconds();
    //                 li /= (e_i - e_j).to_seconds();
    //             }
    //         }
    //         polynomials.0 += x_i * li;
    //         polynomials.1 += y_i * li;
    //         polynomials.2 += z_i * li;
    //     }

    //     Some(polynomials)
    // }
}
