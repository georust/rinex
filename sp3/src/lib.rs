//! SP3 precise orbit file parser.
#![cfg_attr(docsrs, feature(doc_cfg))]
extern crate gnss_rs as gnss;

use itertools::Itertools;

#[cfg(feature = "qc")]
extern crate rinex_qc_traits as qc_traits;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, ParsingError as EpochParsingError, TimeScale};
use map_3d::{ecef2aer, Ellipsoid};
use std::collections::BTreeMap;

use gnss_rs::constellation::ParsingError as ConstellationParsingError;
use thiserror::Error;

#[cfg(feature = "qc")]
mod qc;

#[cfg(feature = "processing")]
mod processing;

#[cfg(feature = "flate2")]
use flate2::bufread::GzDecoder;

#[cfg(test)]
mod tests;

mod header;
mod position;
mod velocity;
mod version;

mod parsing;

#[cfg(docsrs)]
mod bibliography;

use version::Version;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/*
 * 3D position
 */
type Vector3D = (f64, f64, f64);

pub mod prelude {
    pub use crate::{
        version::Version, DataType, Error, OrbitType, ParsingError, SP3Entry, SP3Key, SP3,
    };
    // Pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale};
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    #[default]
    Position,
    Velocity,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Position => f.write_str("P"),
            Self::Velocity => f.write_str("V"),
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else if s.eq("V") {
            Ok(Self::Velocity)
        } else {
            Err(ParsingError::UnknownDataType(s.to_string()))
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    BHN,
    HLM,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FIT => f.write_str("FIT"),
            Self::EXT => f.write_str("EXT"),
            Self::BCT => f.write_str("BCT"),
            Self::BHN => f.write_str("BHN"),
            Self::HLM => f.write_str("HLM"),
        }
    }
}

impl std::str::FromStr for OrbitType {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FIT") {
            Ok(Self::FIT)
        } else if s.eq("EXT") {
            Ok(Self::EXT)
        } else if s.eq("BCT") {
            Ok(Self::BCT)
        } else if s.eq("BHN") {
            Ok(Self::BHN)
        } else if s.eq("HLM") {
            Ok(Self::HLM)
        } else {
            Err(ParsingError::UnknownOrbitType(s.to_string()))
        }
    }
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
    /// Possible clock correction in microsecond with 1E-12 precision.
    /// Often omitted, files that only have orbital states are most common.
    pub clock: Option<f64>,
    /// Clock offset variation, with 0.1ns/s scaling with 1E-16 theoretical precision.
    /// Rarely present.
    pub clock_rate: Option<f64>,
    /// Position vector expressed in the coordinates system, scaling is km and precision
    /// down to 1mm, depending on fit technique.
    pub position: Vector3D,
    /// Possible velocity vector (rarely present) expressed in [km/s] in the same
    /// reference frame.
    pub velocity: Option<Vector3D>,
}

impl SP3Entry {
    /// Builds new [SP3Entry] with given position and all other
    /// fields are unknown.
    pub fn from_position(position: Vector3D) -> Self {
        Self {
            position,
            clock: None,
            velocity: None,
            clock_rate: None,
        }
    }
    /// Builds new [SP3Entry] with given position and velocity vector,
    /// any other fields are unknown
    pub fn from_position_velocity(position: Vector3D, velocity: Vector3D) -> Self {
        Self {
            position,
            clock: None,
            clock_rate: None,
            velocity: Some(velocity),
        }
    }
    /// Copies and returns [Self] with given position vector
    pub fn with_position(&self, position: Vector3D) -> Self {
        let mut s = self.clone();
        s.position = position;
        s
    }
    /// Copies and returns [Self] with given velocity vector
    pub fn with_velocity(&self, velocity: Vector3D) -> Self {
        let mut s = self.clone();
        s.velocity = Some(velocity);
        s
    }
    /// Copies and returns [Self] with given clock offset
    pub fn with_clock_offset(&self, offset: f64) -> Self {
        let mut s = self.clone();
        s.clock = Some(offset);
        s
    }
    /// Copies and returns [Self] with given clock rate
    pub fn with_clock_rate(&self, rate: f64) -> Self {
        let mut s = self.clone();
        s.clock_rate = Some(rate);
        s
    }
}

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3 {
    /// File revision
    pub version: Version,
    /// Data Type used in this file.
    /// If DataType == Velocity, you know
    /// that velocities record will be provided.
    /// Otherwise, that is not garanteed and kind of rare.
    pub data_type: DataType,
    /// Coordinates system used in this file.
    pub coord_system: String,
    /// Type of Orbit contained in this file.
    pub orbit_type: OrbitType,
    /// Agency providing this data
    pub agency: String,
    /// Type of constellations encountered in this file.
    /// For example "GPS" means only GPS vehicles are present.
    pub constellation: Constellation,
    /// [TimeScale] that applies to all following [Epoch]
    pub time_scale: TimeScale,
    /// Initial week counter in [TimeScale]
    pub week_counter: (u32, f64),
    /// Initial MJD, in time_system
    pub mjd_start: (u32, f64),
    /// [`Epoch`]s where at least one position or clock offset is provided
    pub epoch: Vec<Epoch>,
    /// Returns sampling interval, ie., time between successive [`Epoch`]s.
    pub epoch_interval: Duration,
    /// Satellite Vehicles
    pub sv: Vec<SV>,
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

impl SP3 {
    /// Returns a unique Epoch iterator where either
    /// Position or Clock data is provided.
    pub fn epoch(&self) -> impl Iterator<Item = Epoch> + '_ {
        self.epoch.iter().copied()
    }

    /// Returns total number of epoch
    pub fn nb_epochs(&self) -> usize {
        self.epoch.len()
    }

    /// Returns first epoch
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch.first().copied()
    }

    /// Returns last epoch
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch.last().copied()
    }

    /// Returns a unique [Constellation] iterator
    pub fn constellation(&self) -> impl Iterator<Item = Constellation> + '_ {
        self.sv().map(|sv| sv.constellation).unique()
    }

    /// Returns a unique [SV] iterator
    pub fn sv(&self) -> impl Iterator<Item = SV> + '_ {
        self.sv.iter().copied()
    }

    /// Returns an Iterator over SV position estimates, in km with 1mm precision
    /// and expressed in [self.coord_system] (always fixed body frame).
    pub fn sv_position(&self) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.data.iter().map(|(k, v)| (k.epoch, k.sv, v.position))
    }

    /// Returns an Iterator over SV elevation and azimuth angle, both in degrees.
    /// ref_geo: referance position expressed in decimal degrees
    pub fn sv_elevation_azimuth(
        &self,
        ref_geo: Vector3D,
    ) -> impl Iterator<Item = (Epoch, SV, (f64, f64))> + '_ {
        self.sv_position().map(move |(t, sv, (x_km, y_km, z_km))| {
            let (azim, elev, _) = ecef2aer(
                x_km * 1.0E3,
                y_km * 1.0E3,
                z_km * 1.0E3,
                ref_geo.0,
                ref_geo.1,
                ref_geo.2,
                Ellipsoid::WGS84,
            );
            (t, sv, (elev.to_degrees(), azim.to_degrees()))
        })
    }

    /// Returns an Iterator over SV velocities estimates,
    /// in 10^-1 m/s with 0.1 um/s precision.
    pub fn sv_velocities(&self) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let velocity = v.velocity?;
            Some((k.epoch, k.sv, velocity))
        })
    }

    /// Returns an Iterator over Clock offsets with theoretical 1E-12 precision.
    pub fn sv_clock(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let clock = v.clock?;
            Some((k.epoch, k.sv, clock))
        })
    }

    /// Returns an Iterator over Clock offset variations, scaling is 0.1 ns/s and theoretical
    /// precision downto 0.1 fs/s precision.
    pub fn sv_clock_rate(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.data.iter().filter_map(|(k, v)| {
            let rate = v.clock_rate?;
            Some((k.epoch, k.sv, rate))
        })
    }

    /// Interpolate Clock (offset) at desired "t" expressed in the timescale you want.
    /// SP3 files usually have a 15' sampling interval which makes this operation
    /// most likely incorrect. You should either use higher sample rate to reduce
    /// the error generated by interpolation, or use different products like
    /// high precision Clock RINEX files.
    pub fn sv_clock_interpolate(&self, t: Epoch, sv: SV) -> Option<f64> {
        let before = self
            .sv_clock()
            .filter_map(|(clk_t, clk_sv, value)| {
                if clk_t <= t && clk_sv == sv {
                    Some((clk_t, value))
                } else {
                    None
                }
            })
            .last()?;
        let after = self
            .sv_clock()
            .filter_map(|(clk_t, clk_sv, value)| {
                if clk_t > t && clk_sv == sv {
                    Some((clk_t, value))
                } else {
                    None
                }
            })
            .reduce(|k, _| k)?;
        let (before_t, before_clk) = before;
        let (after_t, after_clk) = after;
        let dt = (after_t - before_t).to_seconds();
        let mut bias = (after_t - t).to_seconds() / dt * before_clk;
        bias += (t - before_t).to_seconds() / dt * after_clk;
        Some(bias)
    }

    /// Returns an Iterator over [`Comments`] contained in this file
    pub fn comments_iter(&self) -> impl Iterator<Item = &String> + '_ {
        self.comments.iter()
    }

    /// Interpolates SV position at single instant `t`, results expressed in kilometers
    /// and same reference frame. Typical interpolations vary between 7 and 11,
    /// to preserve the data precision.
    /// For an evenly spaced SP3 file, operation is feasible on Epochs
    /// contained in the interval ](N +1)/2 * τ;  T - (N +1)/2 * τ],
    /// where N is the interpolation order, τ the epoch interval (15 ' is the standard
    /// in SP3) and T the last Epoch in this file. See [Bibliography::Japhet2021].
    pub fn sv_position_interpolate(&self, sv: SV, t: Epoch, order: usize) -> Option<Vector3D> {
        let odd_order = order % 2 > 0;
        let sv_position: Vec<_> = self
            .sv_position()
            .filter_map(|(e, svnn, (x, y, z))| {
                if sv == svnn {
                    Some((e, (x, y, z)))
                } else {
                    None
                }
            })
            .collect();
        /*
         * Determine closest Epoch in time
         */
        let center = match sv_position
            .iter()
            .find(|(e, _)| (*e - t).abs() < self.epoch_interval)
        {
            Some(center) => center,
            None => {
                /*
                 * Failed to determine central Epoch for this SV:
                 * empty data set: should not happen
                 */
                return None;
            },
        };
        // println!("CENTRAL EPOCH : {:?}", center); //DEBUG
        let center_pos = match sv_position.iter().position(|(e, _)| *e == center.0) {
            Some(center) => center,
            None => {
                /* will never happen at this point*/
                return None;
            },
        };

        let (min_before, min_after): (usize, usize) = match odd_order {
            true => ((order + 1) / 2, (order + 1) / 2),
            false => (order / 2, order / 2 + 1),
        };

        if center_pos < min_before || sv_position.len() - center_pos < min_after {
            /* can't design time window */
            return None;
        }

        let mut polynomials = Vector3D::default();
        let offset = center_pos - min_before;

        for i in 0..order + 1 {
            let mut li = 1.0_f64;
            let (e_i, (x_i, y_i, z_i)) = sv_position[offset + i];
            for j in 0..order + 1 {
                let (e_j, _) = sv_position[offset + j];
                if j != i {
                    li *= (t - e_j).to_seconds();
                    li /= (e_i - e_j).to_seconds();
                }
            }
            polynomials.0 += x_i * li;
            polynomials.1 += y_i * li;
            polynomials.2 += z_i * li;
        }

        Some(polynomials)
    }
}
