use thiserror::Error;
use std::str::FromStr;
use crate::antex::frequency::Frequency;
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
/// ATX record is not `epoch` iterable.
/// All `epochs_()` related methods would fail.
pub type Record = Vec<(Antenna, Vec<Frequency>)>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
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
        println!("MARKER \"{}\"", marker);
        if marker.contains("TYPE / SERIAL NO") {
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
            let dazi = content.split_at(20).0;
            if let Ok(dazi) = f64::from_str(dazi) {
                antenna = antenna.with_dazi(dazi)
            }
        }
/*
        } else if marker.eq("ZEN1 / ZEN2 / DZEN") {
            antenna.with_zen()

        } else if marker.eq("VALID FROM") {
            antenna.with_valid_from();

        } else if marker.eq("VALID UNTIL") {
            antenna.with_valid_from();

        } else if marker.eq("SINEX CODE") {
            antenna.with_sinex_code(sinex)

        } else if marker.eq("NORTH / EAST / UP") { 
            let (north, rem) = line.split_at(10);
            let (east, rem) = rem.split_at(10);
            let (up, _) = rem.split_at(10);
            frequency = frequency
                .with_northern(f64::from_str(north)?)
                .with_eastern(f64::from_str(east)?)
                .with_upper(f64::from_str(up)?);
        
        } else if marker.eq("START OF FREQUENCY") {
            frequency = Frequency::default()

        } else if marker.eq("END OF FREQUENCY") {
            frequencues
        }
*/
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
