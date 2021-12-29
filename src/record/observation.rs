use thiserror::Error;
use create::constellation::{Constellation, ConstellationError};

/// ObservationRecord specific (optionnal) marker
const SPECIFIC_RECORD_MARKER: &str = "SYS / # / OBS TYPES";

/// Specific `Errors` descriptor
pub enum Error {

}

/// Specific `Epoch` record
struct EpochRecord {
    // si la ligne commence par >
    //  year (2 digit)
    //  month day hour min
    //  sec
    // epoch flag: 0 = OK
    //             1 = power failure between prev. epoch & self
    // number of sats in current epoch
    // list of PRNs
    // rcvr clock offset
}

/// Specific `Data` record contained in observation files 
struct DataRecord {
    sat_id: u8, // sat PRN
    data: Vec<f64>, // raw phase data
}

impl str::FromStr for DataRecord {
    /// builds `DataRecord` from parsed block content.
    /// Block content should be group of lines for 1 entire packet.
    fn from_str (block: &str, Option<epoch>) -> Result<Self, Self::Err> {
        let mut lines = block.lines();
        let line1_items = lines.next()?
            .split_ascii_whitespace()
            .collect();
        let (sat_id, year, mon, day, h, min, sec) :
            (u8, u16, u16, u16, u16, u16, f32) =
            (u32::from_str_radix(line1_items[0],10)?,
            u8::from_str_radix(line1_items[0],10)?,
            u16::from_str_radix(line1_items[1],10)?,
            u16::from_str_radix(line1_items[2],10)?,
            u16::from_str_radix(line1_items[3],10)?,
            u16::from_str_radix(line1_items[4],10)?,
            u16::from_str_radix(line1_items[5],10)?,
            f32::from_str(line1_items[6])?);
        
        let 
    }
}

pub struct Observation {
    epochs: Option<Vec<Epoch>>,
    data: Vec<DataRecord>,
}

impl std::FromStr for Observation {
    /// Builds Observation from entire record file
    fn from_str (content: &str) -> Observation {
        let lines: Vec<&str> =    
    }
}
