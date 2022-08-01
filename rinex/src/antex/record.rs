use thiserror::Error;
use std::str::FromStr;
use crate::epoch;
use crate::channel;
use crate::antex::frequency::{Frequency, Pattern};
use crate::antex::antenna::{Antenna, Calibration, Method};

/// Returns true if this line matches 
/// the beginning of a `epoch` for ATX file (special files),
/// this is not really an epoch but rather a group of dataset
/// for this given antenna, there is no sampling data attached to it.
pub fn is_new_epoch (content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

/// ANTEX Record content,
/// is a list of Antenna with Several `Frequency` items in it.
/// We do not parse RMS frequencies at the moment, but it will 
/// easily be unlocked in near future.
/// ATX record is not `epoch` iterable, that means, `epochs_xxx()`
/// iteration at the crate level will panic.
pub type Record = Vec<(Antenna, Vec<Frequency>)>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
    #[error("Failed to parse frequency channel")]
    ParseChannelError(#[from] channel::Error),
}

/// Parses entire Antenna block
/// and all inner frequency entries
pub fn build_record_entry (content: &str) -> Result<(Antenna, Vec<Frequency>), Error> {
    let lines = content.lines();
    let mut antenna = Antenna::default();
    let mut frequency = Frequency::default();
    let mut frequencies: Vec<Frequency> = Vec::new();
    for line in lines {
        let (content, marker) = line.split_at(60);
        if marker.contains("START OF ANTENNA") {
            antenna = Antenna::default(); // pointless
                // because we're parsing a single START OF antenna block
                // but it helps the else {} condition
                // at the very bottom, where we consider to be
                // in the Frequency payload
        } else if marker.contains("# OF FREQUENCIES") {
            continue // we don't care about this information,
                // because it can be retrieved with
                // an record.antenna.len() ;)
        } else if marker.contains("END OF ANTENNA") {
            break // end of this block, considered as an `epoch`
                // if we make a parallel with other types of RINEX 

        } else if marker.contains("TYPE / SERIAL NO") {
            let (ant_type, rem) = content.split_at(17);
            let (sn, _) = rem.split_at(20);
            antenna = antenna.with_type(ant_type.trim());
            antenna = antenna.with_serial_num(sn.trim())
        
        } else if marker.contains("METH / BY / # / DATE") {
            let (method, rem) = content.split_at(20);
            let (agency, rem) = rem.split_at(20);
            let (_, rem) = rem.split_at(10); // N#
            let (date, _) = rem.split_at(10);
            let cal = Calibration {
                method: Method::from_str(method.trim()).unwrap(),
                agency: agency.trim().to_string(),
                date: date.trim().to_string(),
            };
            antenna = antenna.with_calibration(cal)
        
        } else if marker.contains("DAZI") {
            let dazi = content.split_at(20).0.trim();
            if let Ok(dazi) = f64::from_str(dazi) {
                antenna = antenna.with_dazi(dazi)
            }
        } else if marker.contains("ZEN1 / ZEN2 / DZEN") {
            let (zen1, rem) = content.split_at(8);
            let (zen2, rem) = rem.split_at(6);
            let (dzen, _) = rem.split_at(6);
            if let Ok(zen1) = f64::from_str(zen1.trim()) {
                if let Ok(zen2) = f64::from_str(zen2.trim()) {
                    if let Ok(dzen) = f64::from_str(dzen.trim()) {
                        antenna = antenna.with_zenith(zen1, zen2, dzen)
                    }
                }
            }

        } else if marker.contains("VALID FROM") {
            let datestr =  content.trim();
            if let Ok(datetime) = epoch::str2date(datestr) {
                antenna = antenna.with_valid_from(datetime)
            }

        } else if marker.contains("VALID UNTIL") {
            let datestr =  content.trim();
            if let Ok(datetime) = epoch::str2date(datestr) {
                antenna = antenna.with_valid_until(datetime)
            }

        } else if marker.contains("SINEX CODE") {
            let sinex = content.split_at(10).0;
            antenna = antenna.with_sinex_code(sinex.trim())

        } else if marker.contains("START OF FREQUENCY") {
            let svnn = content.split_at(10).0;
            let channel = channel::Channel::from_sv_code(svnn.trim())?;
            frequency = Frequency::default()
                .with_channel(channel);
        
        } else if marker.contains("NORTH / EAST / UP") { 
            let (north, rem) = content.split_at(10);
            let (east, rem) = rem.split_at(10);
            let (up, _) = rem.split_at(10);
            if let Ok(north) = f64::from_str(north.trim()) {
                if let Ok(east) = f64::from_str(east.trim()) {
                    if let Ok(up) = f64::from_str(up.trim()) {
                        frequency = frequency
                            .with_northern_eccentricity(north)
                            .with_eastern_eccentricity(east)
                            .with_upper_eccentricity(up)
                    }
                }
            }

        } else if marker.contains("END OF FREQUENCY") {
            frequencies.push(frequency.clone())
        
        } else { // Inside frequency
            // Determine type of pattern
            let (content, marker) = line.split_at(60);
            let (content, rem) = line.split_at(8);
            let values :Vec<f64> = rem
                .split_ascii_whitespace()
                .map(|item| {
                    if let Ok(f) = f64::from_str(item.trim()) {
                        f
                    } else {
                        panic!("failed to \"{}\" \"{}\"", content, marker);
                    }
                })
                .collect();
            if line.contains("NOAZI") {
                frequency = frequency.add_pattern(
                    Pattern::NonAzimuthDependent(values.clone()))
            } else {
                let angle = f64::from_str(content.trim()).unwrap();
                frequency = frequency.add_pattern(
                    Pattern::AzimuthDependent((angle, values.clone())))
            }
        }
    }

    Ok((antenna, frequencies))
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_epoch() {
         let content = "                                                           START OF ANTENNA";
         assert_eq!(is_new_epoch(content), true);
         let content = "TROSAR25.R4      LEIT727259                                 TYPE / SERIAL NO";
         assert_eq!(is_new_epoch(content), false);
         let content = "    26                                                      # OF FREQUENCIES";
         assert_eq!(is_new_epoch(content), false);
         let content = "   G01                                                      START OF FREQUENCY";
         assert_eq!(is_new_epoch(content), false);
    }
}
