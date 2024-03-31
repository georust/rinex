use crate::{
    epoch, merge, merge::Merge, prelude::*, split, split::Split, types::Type, version, Observable,
};

use hifitime::Duration;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

/*
 * Meteo RINEX specific record type.
 */
pub type Record = BTreeMap<Epoch, HashMap<Observable, f64>>;

/*
 * Returns true if given line matches a new Meteo Record Epoch.
 * We use this when browsing a RINEX file, to determine whether
 * we should initiate the parsing of a meteo record entry.
 */
pub(crate) fn is_new_epoch(line: &str, v: version::Version) -> bool {
    if v.major < 3 {
        let min_len = " 15  1  1  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    } else {
        let min_len = " 2021  1  7  0  0  0";
        if line.len() < min_len.len() {
            // minimum epoch descriptor
            return false;
        }
        let datestr = &line[1..min_len.len()];
        epoch::parse_utc(datestr).is_ok() // valid epoch descriptor
    }
}

#[derive(Error, Debug)]
/// Meteo Data `Record` parsing specific errors
pub enum Error {
    #[error("failed to parse epoch")]
    EpochParsingError(#[from] epoch::ParsingError),
    #[error("failed to integer number")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to float number")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

/*
 * Meteo record entry parsing method
 */
pub(crate) fn parse_epoch(
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

/*
 * Epoch formatter
 * is used when we're dumping a Meteo RINEX record entry
 */
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &HashMap<Observable, f64>,
    header: &Header,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    lines.push_str(&format!(
        " {}",
        epoch::format(*epoch, Type::MeteoData, header.version.major)
    ));
    let observables = &header.meteo.as_ref().unwrap().codes;
    let mut index = 0;
    for obscode in observables {
        index += 1;
        if let Some(data) = data.get(obscode) {
            lines.push_str(&format!("{:7.1}", data));
        } else {
            lines.push_str("       ");
        }
        if (index % 8) == 0 {
            lines.push('\n');
        }
    }
    lines.push('\n');
    Ok(lines)
}

impl Merge for Record {
    fn merge(&self, rhs: &Self) -> Result<Self, merge::Error> {
        let mut lhs = self.clone();
        lhs.merge_mut(rhs)?;
        Ok(lhs)
    }
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

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k < &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k >= &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        Ok((r0, r1))
    }
    fn split_dt(&self, _duration: Duration) -> Result<Vec<Self>, split::Error> {
        Ok(Vec::new())
    }
}

#[cfg(feature = "processing")]
use crate::preprocessing::*;

#[cfg(feature = "processing")]
impl Mask for Record {
    fn mask(&self, mask: MaskFilter) -> Self {
        let mut s = self.clone();
        s.mask_mut(mask);
        s
    }
    fn mask_mut(&mut self, mask: MaskFilter) {
        match mask.operand {
            MaskOperand::Equals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e == epoch),
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, data| {
                        data.retain(|code, _| filter.contains(code));
                        !data.is_empty()
                    });
                },
                _ => {},
            },
            MaskOperand::NotEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e != epoch),
                TargetItem::ObservableItem(filter) => {
                    self.retain(|_, data| {
                        data.retain(|code, _| !filter.contains(code));
                        !data.is_empty()
                    });
                },
                _ => {},
            },
            MaskOperand::GreaterEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e >= epoch),
                _ => {},
            },
            MaskOperand::GreaterThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e > epoch),
                _ => {},
            },
            MaskOperand::LowerEquals => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e <= epoch),
                _ => {},
            },
            MaskOperand::LowerThan => match mask.item {
                TargetItem::EpochItem(epoch) => self.retain(|e, _| *e < epoch),
                _ => {},
            },
        }
    }
}

/*
 * Decimates only a given record subset
 */
#[cfg(feature = "processing")]
fn decimate_data_subset(record: &mut Record, subset: &Record, target: &TargetItem) {
    match target {
        TargetItem::ObservableItem(obs_list) => {
            /*
             * Remove given observations where it should now be missing
             */
            for (epoch, observables) in record.iter_mut() {
                if subset.get(epoch).is_none() {
                    // should be missing
                    observables.retain(|observable, _| !obs_list.contains(observable));
                }
            }
        },
        _ => {},
    }
}

#[cfg(feature = "processing")]
impl Decimate for Record {
    fn decimate_by_ratio_mut(&mut self, r: u32) {
        let mut i = 0;
        self.retain(|_, _| {
            let retained = (i % r) == 0;
            i += 1;
            retained
        });
    }
    fn decimate_by_ratio(&self, r: u32) -> Self {
        let mut s = self.clone();
        s.decimate_by_ratio_mut(r);
        s
    }
    fn decimate_by_interval_mut(&mut self, interval: Duration) {
        let mut last_retained = Option::<Epoch>::None;
        self.retain(|e, _| {
            if let Some(last) = last_retained {
                let dt = *e - last;
                if dt > interval {
                    last_retained = Some(*e);
                    true
                } else {
                    false
                }
            } else {
                last_retained = Some(*e);
                true // always retain 1st epoch
            }
        });
    }
    fn decimate_by_interval(&self, interval: Duration) -> Self {
        let mut s = self.clone();
        s.decimate_by_interval_mut(interval);
        s
    }
    fn decimate_match_mut(&mut self, rhs: &Self) {
        self.retain(|e, _| rhs.get(e).is_some());
    }
    fn decimate_match(&self, rhs: &Self) -> Self {
        let mut s = self.clone();
        s.decimate_match_mut(rhs);
        s
    }
}

#[cfg(feature = "processing")]
impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
        match f {
            Filter::Mask(mask) => self.mask_mut(mask),
            Filter::Decimation(filter) => match filter.dtype {
                DecimationType::DecimByRatio(r) => {
                    if filter.target.is_none() {
                        self.decimate_by_ratio_mut(r);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // and decimate
                    let subset = self.mask(mask).decimate_by_ratio(r);

                    // adapt self's subset to new data rates
                    decimate_data_subset(self, &subset, &item);
                },
                DecimationType::DecimByInterval(dt) => {
                    if filter.target.is_none() {
                        self.decimate_by_interval_mut(dt);
                        return; // no need to proceed further
                    }

                    let item = filter.target.unwrap();

                    // apply mask to retain desired subset
                    let mask = MaskFilter {
                        item: item.clone(),
                        operand: MaskOperand::Equals,
                    };

                    // and decimate
                    let subset = self.mask(mask).decimate_by_interval(dt);

                    // adapt self's subset to new data rates
                    decimate_data_subset(self, &subset, &item);
                },
            },
            Filter::Smoothing(_) => todo!("smoothing filter"),
            Filter::Interp(filter) => self.interpolate_mut(filter.series),
        }
    }
}

#[cfg(feature = "processing")]
impl Interpolate for Record {
    fn interpolate(&self, series: TimeSeries) -> Self {
        let mut s = self.clone();
        s.interpolate_mut(series);
        s
    }
    fn interpolate_mut(&mut self, _series: TimeSeries) {
        todo!("meteo:record:interpolate_mut()")
    }
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
