use super::Observable;
use crate::{
    algorithm::Decimation, epoch, gnss_time::TimeScaling, merge, merge::Merge, prelude::*, split,
    split::Split, types::Type, version,
};
use hifitime::Duration;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

/// Meteo RINEX Record content.
/// Data is sorted by [epoch::Epoch] and by Observable.
/// Example of Meteo record browsing and manipulation
/// ```
/// use rinex::*;
/// // grab a METEO RINEX
/// let rnx = Rinex::from_file("../test_resources/MET/V2/abvi0010.15m")
///    .unwrap();
/// // grab record
/// let record = rnx.record.as_meteo()
///    .unwrap();
/// for (epoch, observables) in record.iter() {
///    for (observable, data) in observables.iter() {
///        // Observable is standard 3 letter string code
///        // Data is raw floating point data
///    }
/// }
/// ```
pub type Record = BTreeMap<Epoch, HashMap<Observable, f64>>;

/// Returns true if given line matches a new Meteo Record `epoch`
pub(crate) fn is_new_epoch(line: &str, v: version::Version) -> bool {
    if v.major < 4 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse(datestr).is_ok() // valid epoch descriptor
    }
}

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/// Builds `Record` entry for `MeteoData`
pub fn parse_epoch(
    header: &Header,
    content: &str,
) -> Result<(Epoch, HashMap<Observable, f64>), Error> {
    let mut lines = content.lines();
    let mut line = lines.next().unwrap();

    let mut map: HashMap<Observable, f64> = HashMap::with_capacity(3);

    let mut offset: usize = 18; // YY
    if header.version.major > 2 {
        offset += 2; // YYYY
    }

    let (epoch, _) = epoch::parse(&line[0..offset])?;

    let codes = &header.meteo.as_ref().unwrap().codes;
    let n_codes = codes.len();
    let nb_lines: usize = num_integer::div_ceil(n_codes, 8).into();
    let mut code_index: usize = 0;

    for i in 0..nb_lines {
        for _ in 0..8 {
            let code = &codes[code_index];
            let obs: Option<f64> = match f64::from_str(&line[offset..offset + 7].trim()) {
                Ok(f) => Some(f),
                Err(_) => None,
            };

            if let Some(obs) = obs {
                map.insert(code.clone(), obs);
            }
            code_index += 1;
            if code_index >= n_codes {
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

/// Writes epoch into given streamer
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &HashMap<Observable, f64>,
    header: &Header,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    lines.push_str(&format!(
        " {}",
        epoch::format(*epoch, None, Type::MeteoData, header.version.major)
    ));
    let observables = &header.meteo.as_ref().unwrap().codes;
    let mut index = 0;
    for obscode in observables {
        index += 1;
        if let Some(data) = data.get(obscode) {
            lines.push_str(&format!("{:7.1}", data));
        } else {
            lines.push_str(&format!("       "));
        }
        if (index % 8) == 0 {
            lines.push_str("\n");
        }
    }
    lines.push_str("\n");
    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_epoch() {
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 2, minor: 0 }),
            true
        );
        let content = " 22  1  4  0  0  0  993.4   -6.8   52.9    1.6  337.0    0.0    0.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 2, minor: 0 }),
            true
        );
        let content = " 22  1  4  9 55  0  997.9   -6.4   54.2    2.9  342.0    0.0    0.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 2, minor: 0 }),
            true
        );
        let content = " 22  1  4 10  0  0  997.9   -6.3   55.4    3.4  337.0    0.0    0.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 2, minor: 0 }),
            true
        );
        let content = " 08  1  1  0  0  1 1018.0   25.1   75.9    1.4   95.0    0.0    0.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 2, minor: 0 }),
            true
        );
        let content = " 2021  1  7  0  0  0  993.3   23.0   90.0";
        assert_eq!(
            is_new_epoch(content, version::Version { major: 4, minor: 0 }),
            true
        );
    }
}

impl Merge<Record> for Record {
    /// Merges `rhs` into `Self` without mutable access at the expense of more memcopies
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
    /// Merges `rhs` into `Self`
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), merge::Error> {
        for (epoch, observations) in rhs.iter() {
            if let Some(oobservations) = self.get_mut(epoch) {
                for (observation, data) in observations.iter() {
                    if !oobservations.contains_key(observation) {
                        // new observation
                        oobservations.insert(observation.clone(), *data);
                    }
                }
            } else {
                // new epoch
                self.insert(*epoch, observations.clone());
            }
        }
        Ok(())
    }
}

impl Split<Record> for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k >= &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
}

impl Decimation<Record> for Record {
    /// Decimates Self by desired factor
    fn decim_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    /// Copies and Decimates Self by desired factor
    fn decim_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decim_by_ratio_mut(r);
        s
    }
    /// Decimates Self to fit minimum epoch interval
    fn decim_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained: Option<Epoch> = None;
        self.retain(|e, _| {
            if last_retained.is_some() {
                let dt = *e - last_retained.unwrap();
                last_retained = Some(*e);
                dt > interval
            } else {
                last_retained = Some(*e);
                true // always retain 1st epoch
            }
        });
    }
    /// Copies and Decimates Self to fit minimum epoch interval
    fn decim_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decim_by_interval_mut(interval);
        s
    }
    fn decim_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decim_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decim_match_mut(&rhs);
        s
    }
}

impl TimeScaling<Record> for Record {
    fn convert_timescale(&mut self, ts: TimeScale) {
        self.iter_mut()
            .map(|(k, v)| (k.in_time_scale(ts), v))
            .count();
    }
    fn with_timescale(&self, ts: TimeScale) -> Self {
        let mut s = self.clone();
        s.convert_timescale(ts);
        s
    }
}

/*
use crate::processing::{Filter, FilterItem, MaskFilter, FilterOperand};

impl Filter for Record {
    fn apply(&self, filt: MaskFilter<FilterItem>) -> Self {
        let mut s = self.clone();
        s.apply_mut(filt);
        s
    }
    fn apply_mut(&mut self, filt: MaskFilter<FilterItem>) {
        match filt.operand {
            FilterOperand::Equal => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e,  _| *e == epoch),
                FilterItem::ObservableFilter(filter) => {
                    self.retain(|_, data| {
                        data.retain(|code, _| filter.contains(&code.to_string()));
                        data.len() > 0
                    });
                },
                _ => {},
            },
            FilterOperand::NotEqual => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e,  _| *e != epoch),
                FilterItem::ObservableFilter(filter) => {
                    self.retain(|_, data| {
                        data.retain(|code, _| !filter.contains(&code.to_string()));
                        data.len() > 0
                    });
                },
                _ => {},
            },
            FilterOperand::Above => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e, _| *e >= epoch),
                _ => {},
            },
            FilterOperand::StrictlyAbove => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e, _| *e > epoch),
                _ => {},
            },
            FilterOperand::Below => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e, _| *e <= epoch),
                _ => {},
            },
            FilterOperand::StrictlyBelow => match filt.item {
                FilterItem::EpochFilter(epoch) => self.retain(|e, _| *e < epoch),
                _ => {},
            },
        }
    }
}
*/
