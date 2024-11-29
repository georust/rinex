use num_integer::div_ceil;

use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::{
    epoch::parse_utc as parse_utc_epoch,
    prelude::{Epoch, Header, MeteoKey, Observable, ParsingError, Version},
};

/// Returns true if provided content matches the start of a new Meteo Epoch
pub fn is_new_epoch(line: &str, v: Version) -> bool {
    if v.major < 3 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        parse_utc_epoch(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        parse_utc_epoch(datestr).is_ok() // valid epoch descriptor
    }
}

/// Parses record entries from readable content
/// ## Input
///   - header: [Header] parsed previously
///   - content: readable content
pub fn parse_epoch(header: &Header, content: &str) -> Result<Vec<(MeteoKey, f64)>, ParsingError> {
    let mut lines = content.lines();
    let mut line = lines.next().ok_or(ParsingError::EmptyEpoch)?;
    let mut ret = Vec::<(MeteoKey, f64)>::with_capacity(8);

    let mut offset: usize = 18; // YY
    if header.version.major > 2 {
        offset += 2; // YYYY
    }

    let epoch = parse_utc_epoch(&line[0..offset])?;

    let header = &header
        .meteo
        .as_ref()
        .ok_or(ParsingError::MissingObservableDefinition)?;

    let mut obs_ptr = 0;
    let codes = &header.codes;
    let nb_obs = codes.len();
    let nb_lines_per_obs = div_ceil(nb_obs, 8) as usize;

    for i in 0..nb_lines_per_obs {
        for _ in 0..8 {
            let code = &codes[obs_ptr];
        }
    }

    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 2, minor: 0 }
        ));
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert!(is_new_epoch(
            content,
            version::Version { major: 4, minor: 0 }
        ));
    }
}
