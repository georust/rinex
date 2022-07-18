//! Antex - special RINEX type specific structures
use crate::channel;
use thiserror::Error;
use std::collections::BTreeMap;

pub mod pcv;
pub mod antenna;
pub mod frequency;

use pcv::Pcv;
use antenna::Antenna;
use frequency::Frequency;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
}

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
pub type Record = BTreeMap<Antenna, Vec<Frequency>>;

/// Parses entire Antenna block
/// and all inner frequency entries
pub fn build_record_entry (content: &str) -> Result<(Antenna, Vec<Frequency>), Error> {
    let lines = content.lines();
    let mut antenna = Antenna::default();
    let mut frequency = Frequency::default();
    let mut frequencies: Vec<Frequency> = Vec::new();

    for line in lines {
        let (content, marker) = line.split_at(60);
        let content = content.trim();
        let marker = marker.trim();
/*
        if marker.eq("TYPE / SERIAL NO") {
            antenna.with_serial_num()

        } else if marker.contains("METH / BY") {
            antenna.with_method()

        } else if marker.eq("DAZI") {
            antenna.with_dazi()

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

/// ANTEX special RINEX fields
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Antenna Phase Center Variations type 
    pub pcv: pcv::Pcv, 
    /// Types of relative values, default: "AOAD/M_T"
    pub relative_values: String,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_sn: Option<String>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            pcv: Pcv::default(),
            relative_values: String::new(),
            reference_sn: None,
        }
    }
}
