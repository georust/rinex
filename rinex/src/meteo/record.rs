use crate::{
    epoch, gnss_time::GnssTime, merge, merge::Merge, prelude::*, split,
    split::Split, types::Type, version, Observable,
	processing::{Filter, Preprocessing, MaskOperand, TargetItem},
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
///     for (observable, data) in observables.iter() {
///         if observable == Observable::Temperature {
///             if data > 20.0 { // °C
///             }
///         }
///     }
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
	fn split_dt(&self, duration: Duration) -> Result<Vec<Self>, split::Error> {
		Ok(Vec::new())
	}
}

impl GnssTime<Record> for Record {
	fn timeseries(&self, dt: Duration) -> TimeSeries {
		let epochs: Vec<_> = self.keys().collect();
		TimeSeries::inclusive(
			**epochs.get(0)
				.expect("failed to determine first epoch"),
			**epochs.get(epochs.len()-1)
				.expect("failed to determine last epoch"),
			dt)
	}
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

impl Preprocessing for Record {
    fn filter(&self, f: Filter) -> Self {
        let mut s = self.clone();
        s.filter_mut(f);
        s
    }
    fn filter_mut(&mut self, f: Filter) {
		match f {
			Filter::Mask(mask) => {
				match mask.operand {
					MaskOperand::Equals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e == epoch),
						TargetItem::ObservableItem(filter) => {
							self.retain(|_, data| {
								data.retain(|code, _| filter.contains(code));
								data.len() > 0
							});
						},
						_ => {},
					},
					MaskOperand::NotEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e != epoch),
						TargetItem::ObservableItem(filter) => {
							self.retain(|_, data| {
								data.retain(|code, _| !filter.contains(code));
								data.len() > 0
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
			},
			Filter::Smoothing(_) => todo!(),
			Filter::Decimation(_) => todo!(),
		}
	}
}
