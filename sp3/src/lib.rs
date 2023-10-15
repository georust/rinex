//! SP3 precise orbit file parser.
#![cfg_attr(docrs, feature(doc_cfg))]

extern crate gnss_rs as gnss;

use gnss::prelude::{Constellation, SV};
use hifitime::{Duration, Epoch, TimeScale};
use std::collections::BTreeMap;

use std::str::FromStr;
use thiserror::Error;

#[cfg(test)]
mod tests;

mod header;
mod merge;
mod position;
mod reader;
mod velocity;
mod version;

#[cfg(doc_cfg)]
mod bibliography;

use header::{
    line1::{is_header_line1, Line1},
    line2::{is_header_line2, Line2},
};

use position::{position_entry, ClockRecord, PositionEntry, PositionRecord};
use velocity::{velocity_entry, ClockRateRecord, VelocityEntry, VelocityRecord};

use reader::BufferedReader;
use std::io::BufRead;
use version::Version;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/*
 * 3D position
 */
type Vector3D = (f64, f64, f64);

pub mod prelude {
    pub use crate::version::Version;
    pub use crate::{DataType, OrbitType, SP3};
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale};
}

pub use merge::{Merge, MergeError};

fn file_descriptor(content: &str) -> bool {
    content.starts_with("%c")
}

fn sp3_comment(content: &str) -> bool {
    content.starts_with("/*")
}

fn end_of_file(content: &str) -> bool {
    content.eq("EOF")
}

fn new_epoch(content: &str) -> bool {
    content.starts_with("*  ")
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

/*
 * Comments contained in file
 */
type Comments = Vec<String>;

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
    /// File original time system,
    /// either UTC or time source from which we converted to UTC.
    pub time_system: TimeScale,
    /// Initial week counter, in time_system
    pub week_counter: (u32, f64),
    /// Initial MJD, in time_system
    pub mjd_start: (u32, f64),
    /// [`Epoch`]s where at least one position
    /// or one clock data is provided. Epochs are expressed UTC time,
    /// either directly if provided as such, or internally converted.
    pub epoch: Vec<Epoch>,
    /// Returns sampling interval, ie., time between successive [`Epoch`]s.
    pub epoch_interval: Duration,
    /// Satellite Vehicles
    pub sv: Vec<SV>,
    /// Positions expressed in km, with 1mm precision, per Epoch and SV.
    pub position: PositionRecord,
    /// Clock estimates in microseconds, with 1E-12 precision per Epoch and SV.
    pub clock: ClockRecord,
    /// Velocities (Position derivative estimates) in 10^-1 m/s with 0.1 um/s precision.
    pub velocities: VelocityRecord,
    /// Rate of change of clock correction in 0.1 ns/s with 0.1 fs/s precision.
    pub clock_rate: ClockRateRecord,
    /// File header comments, stored as is.
    pub comments: Comments,
}

#[derive(Debug, Error)]
pub enum Errors {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("hifitime parsing error")]
    HifitimeParsingError(#[from] hifitime::Errors),
    #[error("constellation parsing error")]
    ConstellationParsingError(#[from] gnss::constellation::ParsingError),
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

/*
 * Parses hifitime::Epoch from standard format
 */
fn parse_epoch(content: &str, time_scale: TimeScale) -> Result<Epoch, ParsingError> {
    let y = u32::from_str(content[0..4].trim())
        .or(Err(ParsingError::EpochYear(content[0..4].to_string())))?;

    let m = u32::from_str(content[4..7].trim())
        .or(Err(ParsingError::EpochMonth(content[4..7].to_string())))?;

    let d = u32::from_str(content[7..10].trim())
        .or(Err(ParsingError::EpochDay(content[7..10].to_string())))?;

    let hh = u32::from_str(content[10..13].trim())
        .or(Err(ParsingError::EpochHours(content[10..13].to_string())))?;

    let mm = u32::from_str(content[13..16].trim())
        .or(Err(ParsingError::EpochMinutes(content[13..16].to_string())))?;

    let ss = u32::from_str(content[16..19].trim())
        .or(Err(ParsingError::EpochSeconds(content[16..19].to_string())))?;

    let _ss_fract = f64::from_str(content[20..27].trim()).or(Err(
        ParsingError::EpochMilliSeconds(content[20..27].to_string()),
    ))?;

    Epoch::from_str(&format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
        y, m, d, hh, mm, ss, time_scale,
    ))
    .or(Err(ParsingError::Epoch))
}

impl SP3 {
    /// Parses given SP3 file, with possible seamless
    /// .gz decompression, if compiled with the "flate2" feature.
    pub fn from_file(path: &str) -> Result<Self, Errors> {
        let reader = BufferedReader::new(path)?;

        let mut version = Version::default();
        let mut data_type = DataType::default();

        let mut time_system = TimeScale::default();
        let mut constellation = Constellation::default();
        let mut pc_count = 0_u8;

        let mut coord_system = String::from("Unknown");
        let mut orbit_type = OrbitType::default();
        let mut agency = String::from("Unknown");
        let mut week_counter = (0_u32, 0_f64);
        let mut epoch_interval = Duration::default();
        let mut mjd_start = (0_u32, 0_f64);

        let mut vehicles: Vec<SV> = Vec::new();
        let mut position = PositionRecord::default();
        let mut velocities = VelocityRecord::default();
        let mut clock = ClockRecord::default();
        let mut clock_rate = ClockRateRecord::default();
        let mut comments = Comments::new();

        let mut epoch = Epoch::default();
        let mut epochs: Vec<Epoch> = Vec::new();

        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();
            if sp3_comment(line) {
                if line.len() > 4 {
                    comments.push(line[3..].to_string());
                }
                continue;
            }
            if end_of_file(line) {
                break;
            }
            if is_header_line1(line) && !is_header_line2(line) {
                let l1 = Line1::from_str(line)?;
                (version, data_type, coord_system, orbit_type, agency) = l1.to_parts();
            }
            if is_header_line2(line) {
                let l2 = Line2::from_str(line)?;
                (week_counter, epoch_interval, mjd_start) = l2.to_parts();
            }
            if file_descriptor(line) {
                if line.len() < 60 {
                    return Err(Errors::ParsingError(ParsingError::MalformedDescriptor(
                        line.to_string(),
                    )));
                }

                if pc_count == 0 {
                    constellation = Constellation::from_str(line[3..5].trim())?;
                    time_system = TimeScale::from_str(line[9..12].trim())?;
                }

                pc_count += 1;
            }
            if new_epoch(line) {
                epoch = parse_epoch(&line[3..], time_system)?;
                epochs.push(epoch);
            }
            if position_entry(line) {
                if line.len() < 60 {
                    /*
                     * tolerate malformed positions
                     */
                    continue;
                }
                let entry = PositionEntry::from_str(line)?;
                let (sv, (pos_x, pos_y, pos_z), clk) = entry.to_parts();

                //TODO : move this into %c config frame
                if !vehicles.contains(&sv) {
                    vehicles.push(sv);
                }

                if pos_x != 0.0_f64 && pos_y != 0.0_f64 && pos_z != 0.0_f64 {
                    /*
                     * Position vector is present & correct
                     */
                    if let Some(e) = position.get_mut(&epoch) {
                        e.insert(sv, (pos_x, pos_y, pos_z));
                    } else {
                        let mut map: BTreeMap<SV, Vector3D> = BTreeMap::new();
                        map.insert(sv, (pos_x, pos_y, pos_z));
                        position.insert(epoch, map);
                    }
                }
                if let Some(clk) = clk {
                    /*
                     * Clock data is present & correct
                     */
                    if let Some(e) = clock.get_mut(&epoch) {
                        e.insert(sv, clk);
                    } else {
                        let mut map: BTreeMap<SV, f64> = BTreeMap::new();
                        map.insert(sv, clk);
                        clock.insert(epoch, map);
                    }
                }
            }
            if velocity_entry(line) {
                if line.len() < 60 {
                    /*
                     * tolerate malformed velocities
                     */
                    continue;
                }
                let entry = VelocityEntry::from_str(line)?;
                let (sv, (vel_x, vel_y, vel_z), clk) = entry.to_parts();

                //TODO : move this into %c config frame
                if !vehicles.contains(&sv) {
                    vehicles.push(sv);
                }

                if vel_x != 0.0_f64 && vel_y != 0.0_f64 && vel_z != 0.0_f64 {
                    /*
                     * Position vector is present & correct
                     */
                    if let Some(e) = velocities.get_mut(&epoch) {
                        e.insert(sv, (vel_x, vel_y, vel_z));
                    } else {
                        let mut map: BTreeMap<SV, Vector3D> = BTreeMap::new();
                        map.insert(sv, (vel_x, vel_y, vel_z));
                        velocities.insert(epoch, map);
                    }
                }
                if let Some(clk) = clk {
                    /*
                     * Clock data is present & correct
                     */
                    if let Some(e) = clock_rate.get_mut(&epoch) {
                        e.insert(sv, clk);
                    } else {
                        let mut map: BTreeMap<SV, f64> = BTreeMap::new();
                        map.insert(sv, clk);
                        clock_rate.insert(epoch, map);
                    }
                }
            }
        }
        Ok(Self {
            version,
            data_type,
            epoch: epochs,
            time_system,
            constellation,
            coord_system,
            orbit_type,
            agency,
            week_counter,
            epoch_interval,
            mjd_start,
            sv: vehicles,
            position,
            velocities,
            clock,
            clock_rate,
            comments,
        })
    }
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
        self.epoch.get(0).copied()
    }
    /// Returns last epoch
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch.last().copied()
    }
    /// Returns a unique SV iterator
    pub fn sv(&self) -> impl Iterator<Item = SV> + '_ {
        self.sv.iter().copied()
    }
    /// Returns an Iterator over SV position estimates, in km
    /// with 1mm precision.
    pub fn sv_position(&self) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.position
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, pos)| (*e, *sv, *pos)))
    }
    /// Returns an Iterator over SV elevation and azimuth angle, both in degrees.
    /// ref_geo: referance position expressed in decimal degrees
    pub fn sv_elevation_azimuth(
        &self,
        ref_geo: Vector3D,
    ) -> impl Iterator<Item = (Epoch, SV, (f64, f64))> + '_ {
        self.sv_position().map(move |(t, sv, (x_km, y_km, z_km))| {
            let (azim, elev, _) = map_3d::ecef2aer(
                x_km * 1.0E3,
                y_km * 1.0E3,
                z_km * 1.0E3,
                ref_geo.0,
                ref_geo.1,
                ref_geo.2,
                map_3d::Ellipsoid::WGS84,
            );
            (t, sv, (map_3d::rad2deg(elev), map_3d::rad2deg(azim)))
        })
    }
    /// Returns an Iterator over SV velocities estimates,
    /// in 10^-1 m/s with 0.1 um/s precision.
    pub fn sv_velocities(&self) -> impl Iterator<Item = (Epoch, SV, Vector3D)> + '_ {
        self.velocities
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, vel)| (*e, *sv, *vel)))
    }
    /// Returns an Iterator over Clock error estimates, in microseconds
    /// with 1E-12 precision.
    pub fn sv_clock(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.clock
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, clk)| (*e, *sv, *clk)))
    }
    /// Returns an Iterator over Clock rate of change estimates,
    /// in 0.1 ns/s with 0.1 fs/s precision.
    pub fn sv_clock_change(&self) -> impl Iterator<Item = (Epoch, SV, f64)> + '_ {
        self.clock_rate
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, clk)| (*e, *sv, *clk)))
    }
    /// Returns an Iterator over [`Comments`] contained in this file
    pub fn comments(&self) -> impl Iterator<Item = &String> + '_ {
        self.comments.iter()
    }
    /// Interpolate SV position, expressed in kilometers ECEF at desired Epoch `t`.
    /// For typical SP3 files with 15' epoch interval,
    /// an interpolation order of at least 11 is recommended to preserve
    /// data precision.
    /// For an evenly spaced SP3 file, operation is feasible on Epochs
    /// contained in the interval ](N +1)/2 * τ;  T - (N +1)/2 * τ],
    /// where N is the interpolation order, τ the epoch interval and T
    /// the last Epoch in this file. See [Bibliography::Japhet2021].
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

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        // if self.agency != rhs.agency {
        //     return Err(MergeError::DataProvider);
        // }
        if self.time_system != rhs.time_system {
            return Err(MergeError::TimeScale);
        }
        if self.coord_system != rhs.coord_system {
            return Err(MergeError::CoordSystem);
        }
        if self.constellation != rhs.constellation {
            /*
             * Convert self to Mixed constellation
             */
            self.constellation = Constellation::Mixed;
        }
        // adjust revision
        if rhs.version > self.version {
            self.version = rhs.version;
        }
        // Adjust MJD start
        if rhs.mjd_start.0 < self.mjd_start.0 {
            self.mjd_start.0 = rhs.mjd_start.0;
        }
        if rhs.mjd_start.1 < self.mjd_start.1 {
            self.mjd_start.1 = rhs.mjd_start.1;
        }
        // Adjust week counter
        if rhs.week_counter.0 < self.week_counter.0 {
            self.week_counter.0 = rhs.week_counter.0;
        }
        if rhs.week_counter.1 < self.week_counter.1 {
            self.week_counter.1 = rhs.week_counter.1;
        }
        // update SV table
        for sv in &rhs.sv {
            if !self.sv.contains(sv) {
                self.sv.push(*sv);
            }
        }
        // update sampling interval (pessimistic)
        self.epoch_interval = std::cmp::max(self.epoch_interval, rhs.epoch_interval);
        /*
         * Merge possible new positions
         */
        for (epoch, svnn) in &rhs.position {
            if let Some(lhs_sv) = self.position.get_mut(epoch) {
                for (sv, position) in svnn {
                    lhs_sv.insert(*sv, *position);
                }
            } else {
                // introduce new epoch
                self.epoch.push(*epoch);
                self.position.insert(*epoch, svnn.clone());
            }
        }
        /*
         * Merge possible new Clock estimates
         */
        for (epoch, svnn) in &rhs.clock {
            if let Some(lhs_sv) = self.clock.get_mut(epoch) {
                for (sv, clock) in svnn {
                    lhs_sv.insert(*sv, *clock);
                }
            } else {
                // introduce new epoch : in clock record
                self.clock.insert(*epoch, svnn.clone());
                // introduce new epoch : if not contained in positions
                let mut found = false;
                for e in &self.epoch {
                    found |= *e == *epoch;
                    if found {
                        break;
                    }
                }
                if !found {
                    self.epoch.push(*epoch);
                }
            }
        }
        /*
         * Merge possible new Velocities estimates
         */
        for (epoch, svnn) in &rhs.velocities {
            if let Some(lhs_sv) = self.velocities.get_mut(epoch) {
                for (sv, position) in svnn {
                    lhs_sv.insert(*sv, *position);
                }
            } else {
                // introduce new epoch
                self.velocities.insert(*epoch, svnn.clone());
                // introduce new epoch : if not contained in positions
                let mut found = false;
                for e in &self.epoch {
                    found |= *e == *epoch;
                    if found {
                        break;
                    }
                }
                if !found {
                    self.epoch.push(*epoch);
                }
            }
        }

        // maintain Epochs in correct order
        self.epoch.sort();
        Ok(())
    }
}
