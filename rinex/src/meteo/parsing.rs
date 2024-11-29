use crate::{epoch::parse_utc as parse_utc_epoch, prelude::Version};

use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
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
pub fn parse_epoch(header: &Header, content: &str) -> Result<Vec<MeteoKey, f64>> {
    let mut lines = content.lines();

    let mut line = lines.next().ok_or(ParsingError::EmptyEpoch)?;

    let mut map: HashMap<Observable, f64> = HashMap::with_capacity(3);

    let mut offset: usize = 18; // YY
    if header.version.major > 2 {
        offset += 2; // YYYY
    }

    let epoch = epoch::parse_utc(&line[0..offset])?;

    let codes = &header.meteo.as_ref().unwrap().codes;
    let nb_codes = codes.len();
    let nb_lines: usize = num_integer::div_ceil(nb_codes, 8);
    let mut code_index: usize = 0;

    for i in 0..nb_lines {
        for _ in 0..8 {
            let code = &codes[code_index];
            let obs: Option<f64> = match f64::from_str(line[offset..offset + 7].trim()) {
                Ok(f) => Some(f),
                Err(_) => None,
            };

            if let Some(obs) = obs {
                map.insert(code.clone(), obs);
            }
            code_index += 1;
            if code_index >= nb_codes {
                break;
            }

            offset += 7;
            if offset >= line.len() {
                break;
            }
        } // 1:8

        if i < nb_lines - 1 {
            if let Some(l) = lines.next() {
                line = l;
            } else {
                break;
            }
        }
    } // nb lines
    Ok((epoch, map))
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
