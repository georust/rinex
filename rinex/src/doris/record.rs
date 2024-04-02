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
/// Measurements are stored by Kind, by Station and by TAI sampling instant.
pub type Record = BTreeMap<(Epoch, EpochFlag), BTreeMap<Station, HashMap<Observable, f64>>>;

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
) -> Result<
    (
        (Epoch, EpochFlag),
        BTreeMap<Station, HashMap<Observable, f64>>,
    ),
    Error,
> {
    let mut epoch = Epoch::default();
    let mut flag = EpochFlag::default();
    let mut buffer = BTreeMap::<Station, HashMap<Observable, f64>>::new();

    let doris = header
        .doris
        .as_ref()
        .expect("missing header field(s): badly formed DORIS RINEX");

    let observables = &doris.observables;
    let stations = &doris.stations;
    let mut obs_idx = 0usize;

    assert!(
        stations.len() > 0,
        "badly formed DORIS RINEX: no stations defined"
    );
    let mut station = Option::<Station>::None;

    for (lindex, line) in content.lines().enumerate() {
        match lindex {
            0 => {
                /* 1st line gives TAI timestamp, flag, clock offset */
                let line = line.split_at(2).1; // "> "
                let offset = "YYYY MM DD HH MM SS.NNNNNNNNN  0".len();
                let (date, rem) = line.split_at(offset);
                (epoch, flag) = parse_in_timescale(date, TimeScale::TAI)?;
            },
            _ => {
                let mut iter = line.split_ascii_whitespace();

                if obs_idx == 0 {
                    // parse station identifier
                    let id = iter
                        .next()
                        .expect("missing station identifier: badly formed DORIS RINEX");
                    assert!(id.len() > 1, "badly formed DORIS station identifier");
                    let key = &id[1..];
                    let key = key
                        .parse::<u16>()
                        .unwrap_or_else(|e| panic!("failed to identify DORIS station: {:?}", e));

                    station = stations
                        .iter()
                        .filter(|station| station.key == key)
                        .reduce(|k, _| k)
                        .cloned();
                }

                let identified_station =
                    station.as_ref().expect("failed to identify DORIS station");

                // consume this line
                while let Some(content) = iter.next() {
                    let value = content
                        .parse::<f64>()
                        .unwrap_or_else(|e| panic!("failed to parse float value: {:?}", e));

                    let observable = observables.get(obs_idx).unwrap_or_else(|| {
                        panic!(
                            "failed to determine observable for {:?}({:?})",
                            identified_station, epoch
                        )
                    });

                    if let Some(station) = buffer.get_mut(identified_station) {
                        station.insert(observable.clone(), value);
                    } else {
                        let mut inner =
                            HashMap::from_iter([(Observable::default(), value)].into_iter());
                        buffer.insert(identified_station.clone(), inner);
                    }

                    obs_idx += 1;
                }

                if obs_idx == observables.len() {
                    obs_idx = 0;
                    station = None;
                }
            },
        }
    }
    Ok(((epoch, flag), buffer))
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
