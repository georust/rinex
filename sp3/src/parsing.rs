
use std::{
    io::{
        Read, BufRead, BufReader,
    },
    path::Path,
    fs::File,
    collections::BTreeMap,
    str::FromStr,
};

#[cfg(feature = "flate2")]
use flate2::read::GzDecoder;

use crate::{
    header::{
        line1::{is_header_line1, Line1},
        line2::{is_header_line2, Line2},
    },
    position::{position_entry, PositionEntry},
    velocity::{velocity_entry, VelocityEntry},
    prelude::{SP3, Constellation, Error, Version, DataType, TimeScale, OrbitType, Duration, SV, SP3Key, SP3Entry, Epoch, ParsingError},
};

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

/// Parses [Epoch] from standard SP3 format
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

    /// Parse [SP3] data from local file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let fd = File::open(path).expect("file is not readable");
        let mut reader = BufReader::new(fd);
        Self::from_reader(&mut reader)
    }

    #[cfg(feature = "flate2")]
    /// Parse [SP3] data from gzip encoded local file.
    pub fn from_gzip_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let fd = File::open(path).expect("file is not readable");
        let fd = GzDecoder::new(fd);
        let mut reader = BufReader::new(fd);
        Self::from_reader(&mut reader)
    }
    
    /// Parse [SP3] data from [Read]able I/O.
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Result<Self, Error> {

        let mut version = Version::default();
        let mut data_type = DataType::default();

        let mut time_scale = TimeScale::default();
        let mut constellation = Constellation::default();
        let mut pc_count = 0_u8;

        let mut coord_system = String::from("Unknown");
        let mut orbit_type = OrbitType::default();
        let mut agency = String::from("Unknown");
        let mut week_counter = (0_u32, 0_f64);
        let mut epoch_interval = Duration::default();
        let mut mjd_start = (0_u32, 0_f64);

        let mut vehicles: Vec<SV> = Vec::new();
        let mut comments = Vec::new();
        let mut data = BTreeMap::<SP3Key, SP3Entry>::new();

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
                    return Err(Error::ParsingError(ParsingError::MalformedDescriptor(
                        line.to_string(),
                    )));
                }

                if pc_count == 0 {
                    constellation = Constellation::from_str(line[3..5].trim())?;
                    time_scale = TimeScale::from_str(line[9..12].trim())?;
                }

                pc_count += 1;
            }
            if new_epoch(line) {
                epoch = parse_epoch(&line[3..], time_scale)?;
                epochs.push(epoch);
            }
            if position_entry(line) {
                if line.len() < 60 {
                    continue; // tolerates malformed positions
                }
                let entry = PositionEntry::from_str(line)?;
                let (sv, (x_km, y_km, z_km), clk) = entry.to_parts();

                //TODO : move this into %c config frame
                if !vehicles.contains(&sv) {
                    vehicles.push(sv);
                }
                // verify entry validity
                if x_km != 0.0_f64 && y_km != 0.0_f64 && z_km != 0.0_f64 {
                    let key = SP3Key { epoch, sv };
                    if let Some(e) = data.get_mut(&key) {
                        e.position = (x_km, y_km, z_km);
                    } else {
                        if let Some(clk) = clk {
                            data.insert(
                                key,
                                SP3Entry::from_position((x_km, y_km, z_km)).with_clock_offset(clk),
                            );
                        } else {
                            data.insert(key, SP3Entry::from_position((x_km, y_km, z_km)));
                        }
                    }
                }
            }
            if velocity_entry(line) {
                if line.len() < 60 {
                    continue; // tolerates malformed velocities
                }
                let entry = VelocityEntry::from_str(line)?;
                let (sv, (vel_x, vel_y, vel_z), clk) = entry.to_parts();

                //TODO : move this into %c config frame
                if !vehicles.contains(&sv) {
                    vehicles.push(sv);
                }
                // verify entry validity
                if vel_x != 0.0_f64 && vel_y != 0.0_f64 && vel_z != 0.0_f64 {
                    let key = SP3Key { epoch, sv };
                    if let Some(e) = data.get_mut(&key) {
                        *e = e.with_velocity((vel_x, vel_y, vel_z));
                        if let Some(clk) = clk {
                            *e = e.with_clock_rate(clk);
                        }
                    } else {
                        if let Some(clk) = clk {
                            data.insert(
                                key,
                                SP3Entry::from_position((0.0, 0.0, 0.0)).with_clock_rate(clk),
                            );
                        } else {
                            data.insert(key, SP3Entry::from_position((0.0, 0.0, 0.0)));
                        }
                    }
                }
            }
        }
        Ok(Self {
            version,
            data_type,
            epoch: epochs,
            time_scale,
            constellation,
            coord_system,
            orbit_type,
            agency,
            week_counter,
            epoch_interval,
            mjd_start,
            sv: vehicles,
            data,
            comments,
        })
    }
}