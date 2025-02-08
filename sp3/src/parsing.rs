use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader, Read},
    path::Path,
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
    prelude::{
        Constellation, Epoch, Error, Header, ParsingError, SP3Entry, SP3Key, TimeScale, SP3, SV,
    },
    velocity::{velocity_entry, VelocityEntry},
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
fn parse_epoch(content: &str, timescale: TimeScale) -> Result<Epoch, ParsingError> {
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
        y, m, d, hh, mm, ss, timescale,
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
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    /// Parse [SP3] data from gzip encoded local file.
    pub fn from_gzip_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let fd = File::open(path).expect("file is not readable");
        let fd = GzDecoder::new(fd);
        let mut reader = BufReader::new(fd);
        Self::from_reader(&mut reader)
    }

    /// Parse [SP3] data from [Read]able I/O.
    pub fn from_reader<R: Read>(reader: &mut BufReader<R>) -> Result<Self, Error> {
        let mut pc_count = 0_u8;
        let mut header = Header::default();
        let mut timescale = TimeScale::default();

        let mut vehicles: Vec<SV> = Vec::new();
        let mut comments = Vec::new();
        let mut data = BTreeMap::<SP3Key, SP3Entry>::new();

        let mut epoch = Epoch::default();

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
                let (version, data_type, coord_system, orbit_type, agency) = l1.to_parts();
                header.version = version;
                header.data_type = data_type;
                header.coord_system = coord_system;
                header.orbit_type = orbit_type;
                header.agency = agency;
            }

            if is_header_line2(line) {
                let l2 = Line2::from_str(line)?;
                let ((week_counter, week_sow), epoch_interval, (mjd_int, mjd_fract)) =
                    l2.to_parts();

                header.week_counter = week_counter;
                header.week_sow = week_sow;

                header.epoch_interval = epoch_interval;

                header.mjd = mjd_int as f64;
                header.mjd += mjd_fract;
            }

            if file_descriptor(line) {
                if line.len() < 60 {
                    return Err(Error::ParsingError(ParsingError::MalformedDescriptor(
                        line.to_string(),
                    )));
                }

                if pc_count == 0 {
                    header.constellation = Constellation::from_str(line[3..5].trim())?;
                    timescale = TimeScale::from_str(line[9..12].trim())?;

                    header.timescale = timescale;
                }

                pc_count += 1;
            }

            if new_epoch(line) {
                epoch = parse_epoch(&line[3..], timescale)?;
            }

            if position_entry(line) {
                if line.len() < 60 {
                    // tolerates malformed position vectors
                    continue;
                }

                let entry = PositionEntry::from_str(line)?;

                //TODO : move this into %c config frame
                if !vehicles.contains(&entry.sv) {
                    vehicles.push(entry.sv);
                }

                // verify entry validity
                if entry.x_km != 0.0_f64 && entry.y_km != 0.0_f64 && entry.z_km != 0.0_f64 {
                    let key = SP3Key {
                        epoch,
                        sv: entry.sv,
                    };

                    if let Some(e) = data.get_mut(&key) {
                        e.position_km = (entry.x_km, entry.y_km, entry.z_km);
                        e.orbit_prediction = entry.orbit_prediction;
                    } else {
                        if let Some(clk_us) = entry.clock_us {
                            let value = if entry.orbit_prediction {
                                SP3Entry::from_predicted_position_km((
                                    entry.x_km, entry.y_km, entry.z_km,
                                ))
                            } else {
                                SP3Entry::from_position_km((entry.x_km, entry.y_km, entry.z_km))
                            };

                            let value = if entry.clock_prediction {
                                value.with_predicted_clock_offset_us(clk_us)
                            } else {
                                value.with_clock_offset_us(clk_us)
                            };

                            data.insert(key, value);
                        } else {
                            let value = if entry.orbit_prediction {
                                SP3Entry::from_predicted_position_km((
                                    entry.x_km, entry.y_km, entry.z_km,
                                ))
                            } else {
                                SP3Entry::from_position_km((entry.x_km, entry.y_km, entry.z_km))
                            };

                            data.insert(key, value);
                        }
                    }
                }
            }

            if velocity_entry(line) {
                if line.len() < 60 {
                    // tolerates malformed velocity vectors
                    continue;
                }

                let entry = VelocityEntry::from_str(line)?;
                let (sv, (vel_x_dm_s, vel_y_dm_s, vel_z_dm_s), clk_sub_ns) = entry.to_parts();

                let (vel_x_km_s, vel_y_km_s, vel_z_km_s) = (
                    vel_y_dm_s * 1.0E-4,
                    vel_y_dm_s * 1.0E-4,
                    vel_z_dm_s * 1.0E-4,
                );

                //TODO : move this into %c config frame
                if !vehicles.contains(&sv) {
                    vehicles.push(sv);
                }

                // verify entry validity
                if vel_x_dm_s != 0.0_f64 && vel_y_dm_s != 0.0_f64 && vel_z_dm_s != 0.0_f64 {
                    let key = SP3Key { epoch, sv };
                    if let Some(e) = data.get_mut(&key) {
                        *e = e.with_velocity_km_s((vel_x_km_s, vel_y_km_s, vel_z_km_s));

                        if let Some(clk_sub_ns) = clk_sub_ns {
                            *e = e.with_clock_drift_ns(clk_sub_ns * 0.1);
                        }
                    } else {
                        // Entry does not exist (velocity prior position)
                        // Should not exist, but we tolerate
                        if let Some(clk_sub_ns) = clk_sub_ns {
                            data.insert(
                                key,
                                SP3Entry::from_position_km((0.0, 0.0, 0.0))
                                    .with_velocity_km_s((vel_x_km_s, vel_y_km_s, vel_z_km_s))
                                    .with_clock_drift_ns(clk_sub_ns * 0.1),
                            );
                        } else {
                            data.insert(
                                key,
                                SP3Entry::from_position_km((0.0, 0.0, 0.0))
                                    .with_velocity_km_s((vel_x_km_s, vel_y_km_s, vel_z_km_s)),
                            );
                        }
                    }
                }
            }
        }
        Ok(Self {
            header,
            data,
            comments,
        })
    }
}
