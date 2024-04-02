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
                panic!("DATE: \"{}\", {:?}({})", date, epoch, flag);
            },
            _ => {
                /* others are actual measurements */
                println!("\"{}\"", line);
            },
        }
    }
    panic!("done");
}

#[cfg(test)]
mod test {
    use super::is_new_epoch;
    use crate::Header;
    #[test]
    fn new_epoch() {
        for (desc, expected) in [
            (
                "> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "> 2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                true,
            ),
            (
                "  2023 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            (
                "  2022 01 01 00 00 33.999947700  0  2       -0.151364695 0 ",
                false,
            ),
            ("test", false),
        ] {
            assert_eq!(is_new_epoch(desc), expected);
        }
    }
    use super::parse_epoch;
    #[test]
    fn valid_epoch() {
        let header = Header::default();
        for desc in ["> 2024 01 01 00 00 28.999947700  0  2       -0.151364695 0 
D01  -3237877.052    -2291024.044    21903595.62311  21903633.08011      -113.100 7
          -98.400 7       437.801        1002.000 1       -20.000 1        82.000 1
D02  -2069899.788     -407871.014     4677242.25714   4677392.20614      -119.050 7
         -111.000 7       437.801        1007.000 0        -2.000 0        74.000 0"]
        {
            let epoch = parse_epoch(&header, desc);
            assert!(epoch.is_ok(), "failed to parse DORIS epoch");
        }
    }
}
