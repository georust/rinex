use hifitime::Epoch;
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

use crate::{
    doris::Station,
    epoch::{parse_in_timescale, EpochFlag, ParsingError as EpochParsingError},
    header::Header,
    observable::Observable,
    prelude::TimeScale,
};

/// DORIS measurement parsing error
/// DORIS RINEX Record content.
/// Measurements are stored by Station and by TAI instant.
pub type Record = BTreeMap<(Epoch, EpochFlag), HashMap<Station, f64>>;

/// Returns true if following line matches a new DORIS measurement
pub(crate) fn is_new_epoch(line: &str) -> bool {
    line.starts_with('>')
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] EpochParsingError),
    #[error("failed to parse data")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

#[cfg(feature = "serde")]
use serde::Serialize;

/// DORIS measurement parsing process
pub(crate) fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<((Epoch, EpochFlag), HashMap<Station, f64>), Error> {
    let mut map = HashMap::<Station, f64>::new();
    for (lindex, line) in content.lines().enumerate() {
        match lindex {
            0 => {
                /* 1st line gives TAI timestamp, flag, clock offset */
                let line = line.split_at(2).1; // "> "
                let offset = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
                let (date, rem) = line.split_at(offset);
                let (epoch, flag) = parse_in_timescale(date, TimeScale::TAI)?;
                panic!("DATE: \"{}\", {:?}:{}", date, epoch, flag);
                println!("REM: \"{}\"", rem);
            },
            _ => {
                /* others are actual measurements */
                println!("\"{}\"", line);
            },
        }
    }
    panic!("done");
}
