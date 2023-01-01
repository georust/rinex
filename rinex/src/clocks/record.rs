use crate::{
    epoch, gnss_time::TimeScaling, merge, merge::Merge, prelude::*, split,
    split::Split, version::Version,
	processing::{Filter, Preprocessing, MaskOperand, TargetItem},
};
use hifitime::Duration;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use strum_macros::EnumString;
use thiserror::Error;

#[derive(Error, PartialEq, Eq, Hash, Clone, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum System {
    /// Sv system for AS data
    Sv(Sv),
    /// Stations or Receiver name for other types of data
    Station(String),
}

impl Default for System {
    fn default() -> Self {
        Self::Station(String::from("Unknown"))
    }
}

impl System {
    /// Unwraps self as a `satellite vehicule`
    pub fn as_sv(&self) -> Option<Sv> {
        match self {
            System::Sv(s) => Some(*s),
            _ => None,
        }
    }
    /// Unwraps self as a `station` identification code
    pub fn as_station(&self) -> Option<String> {
        match self {
            System::Station(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(sv) = self.as_sv() {
            f.write_str(&sv.to_string())?
        } else if let Some(station) = self.as_station() {
            f.write_str(&station)?
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
/// Clocks file parsing & identification errors
pub enum Error {
    #[error("unknown data code \"{0}\"")]
    UnknownDataCode(String),
    #[error("failed to parse epoch")]
    EpochError(#[from] epoch::Error),
    #[error("failed to parse # of data fields")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse data payload")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify observable")]
    ParseObservableError(#[from] strum::ParseError),
    #[error("failed to write data")]
    WriterIoError(#[from] std::io::Error),
}

/// Clocks file payload
#[derive(Clone, Debug, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Data {
    /// Clock bias
    pub bias: f64,
    pub bias_sigma: Option<f64>,
    pub rate: Option<f64>,
    pub rate_sigma: Option<f64>,
    pub accel: Option<f64>,
    pub accel_sigma: Option<f64>,
}

/// Clock data observables
#[derive(Debug, PartialEq, Eq, Hash, Clone, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// Data analysis results for receiver clocks
    /// derived from a set of network receivers and satellites
    AR,
    /// Data analysis results for satellites clocks
    /// derived from a set of network receivers and satellites
    AS,
    /// Calibration measurements for a single GNSS receiver
    CR,
    /// Discontinuity measurements for a single GNSS receiver
    DR,
    /// Monitor measurements for the broadcast sallite clocks
    MS,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AR => f.write_str("AR"),
            Self::AS => f.write_str("AS"),
            Self::CR => f.write_str("CR"),
            Self::DR => f.write_str("DR"),
            Self::MS => f.write_str("MS"),
        }
    }
}

/// Clocks RINEX record content.
/// RINEX record for CLOCKS files,
/// Data is sorted by [epoch::Epoch], by [DataType] and by [System].
/* TODO
/// Example of Clock record browsing:
/// ```
/// use rinex::*;
/// // grab a Clock RINEX
/// let rnx = Rinex::from_file("../../test_resources/CLK/V2/COD20352.CLK")
///    .unwrap();
/// // grab record
/// let record = rnx.record.as_clock()
///    .unwrap();
/// for (epoch, datatypes) in record.iter() {
///    for (datatype, systems) in datatypes.iter() {
///       for (system, data) in systems.iter() {
///       }
///    }
/// }
/// ```
*/
pub type Record = BTreeMap<Epoch, HashMap<DataType, HashMap<System, Data>>>;

pub(crate) fn is_new_epoch(line: &str) -> bool {
    // first 2 bytes match a DataType code
    let content = line.split_at(2).0;
    DataType::from_str(content).is_ok()
}

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub(crate) fn parse_epoch(
    version: Version,
    content: &str,
) -> Result<(Epoch, DataType, System, Data), Error> {
    let mut lines = content.lines();
    let line = lines.next().unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let data_type = DataType::from_str(dtype.trim())?; // must pass
    let mut rem = rem.clone();
    let limit = Version {
        major: 3,
        minor: 04,
    };

    let system: System = match version < limit {
        true => {
            // old fashion
            let (system_str, r) = rem.split_at(5);
            rem = r.clone();
            if let Ok(svnn) = Sv::from_str(system_str.trim()) {
                System::Sv(svnn)
            } else {
                System::Station(system_str.trim().to_string())
            }
        },
        false => {
            // modern fashion
            let (system_str, r) = rem.split_at(4);
            if let Ok(svnn) = Sv::from_str(system_str.trim()) {
                let (_, r) = r.split_at(6);
                rem = r.clone();
                System::Sv(svnn)
            } else {
                let mut content = system_str.to_owned();
                let (remainder, r) = r.split_at(6);
                rem = r.clone();
                content.push_str(remainder);
                System::Station(content.trim().to_string())
            }
        },
    };

    // Epoch
    let offset = 4+1 // Y always a 4 digit number, even on RINEX2
       +2+1 // m
       +2+1  // d
       +2+1  // h
       +2+1  // m
        +11; // s
    let (epoch, rem) = rem.split_at(offset);
    let (epoch, _) = epoch::parse(epoch.trim())?;

    // nb of data fields
    let (n, _) = rem.split_at(4);
    let n = u8::from_str_radix(n.trim(), 10)?;

    // data fields
    let mut data = Data::default();
    let items: Vec<&str> = line.split_ascii_whitespace().collect();
    data.bias = f64::from_str(items[9].trim())?; // bias must pass
    if n > 1 {
        if let Ok(f) = f64::from_str(items[10].trim()) {
            data.bias_sigma = Some(f)
        }
    }

    if n > 2 {
        if let Some(l) = lines.next() {
            let line = l.clone();
            let items: Vec<&str> = line.split_ascii_whitespace().collect();
            for i in 0..items.len() {
                if let Ok(f) = f64::from_str(items[i].trim()) {
                    if i == 0 {
                        data.rate = Some(f);
                    } else if i == 1 {
                        data.rate_sigma = Some(f);
                    } else if i == 2 {
                        data.accel = Some(f);
                    } else if i == 3 {
                        data.accel_sigma = Some(f);
                    }
                }
            }
        }
    }

    Ok((epoch, data_type, system, data))
}

/// Writes epoch into stream
pub(crate) fn fmt_epoch(
    epoch: &Epoch,
    data: &HashMap<DataType, HashMap<System, Data>>,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (dtype, data) in data.iter() {
        for (system, data) in data.iter() {
            lines.push_str(&format!("{} {} {} ", dtype, system, epoch));
            lines.push_str(&format!("{:.13E} ", data.bias));
            if let Some(sigma) = data.bias_sigma {
                lines.push_str(&format!("{:.13E} ", sigma));
            }
            if let Some(rate) = data.rate {
                lines.push_str(&format!("{:.13E} ", rate));
            }
            if let Some(sigma) = data.rate_sigma {
                lines.push_str(&format!("{:.13E} ", sigma));
            }
            if let Some(accel) = data.accel {
                lines.push_str(&format!("{:.13E} ", accel));
            }
            if let Some(sigma) = data.accel_sigma {
                lines.push_str(&format!("{:.13E} ", sigma));
            }
            lines.push_str("\n");
        }
    }
    Ok(lines)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_new_epoch() {
        let c = "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), true);
        let c = "RA AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), false);
        let c = "DR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), true);
        let c = "CR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), true);
        let c = "AS AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), true);
        let c =
            "CR USNO      1995 07 14 20 59 50.000000  2    0.123456789012E+00  -0.123456789012E-01";
        assert_eq!(is_new_epoch(c), true);
        let c = "AS G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), true);
        let c = "A  G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01";
        assert_eq!(is_new_epoch(c), false);
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
        for (epoch, dtypes) in rhs.iter() {
            if let Some(ddtypes) = self.get_mut(epoch) {
                for (dtype, systems) in dtypes.iter() {
                    if let Some(ssystems) = ddtypes.get_mut(dtype) {
                        for (system, data) in systems.iter() {
                            if let Some(ddata) = ssystems.get_mut(system) {
                                // provide only previously omitted fields
                                if let Some(data) = data.bias_sigma {
                                    if ddata.bias_sigma.is_none() {
                                        ddata.bias_sigma = Some(data);
                                    }
                                }
                                if let Some(data) = data.rate {
                                    if ddata.rate.is_none() {
                                        ddata.rate = Some(data);
                                    }
                                }
                                if let Some(data) = data.rate_sigma {
                                    if ddata.rate_sigma.is_none() {
                                        ddata.rate_sigma = Some(data);
                                    }
                                }
                                if let Some(data) = data.accel {
                                    if ddata.accel.is_none() {
                                        ddata.accel = Some(data);
                                    }
                                }
                                if let Some(data) = data.accel_sigma {
                                    if ddata.accel_sigma.is_none() {
                                        ddata.accel_sigma = Some(data);
                                    }
                                }
                            } else {
                                // new system
                                ssystems.insert(system.clone(), data.clone());
                            }
                        }
                    } else {
                        //new data type
                        ddtypes.insert(dtype.clone(), systems.clone());
                    }
                }
            } else {
                // new epoch
                self.insert(*epoch, dtypes.clone());
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
                if k <= &epoch {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k > &epoch {
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
						TargetItem::ConstellationItem(mask) => {
							self.retain(|_, dtypes| {
								dtypes.retain(|_, systems| {
									systems.retain(|system, _| {
										if let Some(sv) = system.as_sv() {
											mask.contains(&sv.constellation)
										} else {
											true // retain other system types
										}
									});
									systems.len() > 0
								});
								dtypes.len() > 0
							});
						},
						_ => {}, // TargetItem::
					},
					MaskOperand::NotEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e != epoch),
						_ => {}, // TargetItem::
					},
					MaskOperand::GreaterEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e >= epoch),
						_ => {}, // TargetItem::
					},
					MaskOperand::GreaterThan => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e > epoch),
						_ => {}, // TargetItem::
					},
					MaskOperand::LowerEquals => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e <= epoch),
						_ => {}, // TargetItem::
					},
					MaskOperand::LowerThan => match mask.item {
						TargetItem::EpochItem(epoch) => self.retain(|e, _| *e < epoch),
						_ => {}, // TargetItem::
					},
				}
			},
			Filter::Smoothing(_) => todo!(),
			Filter::Decimation(_) => todo!(),
		}
	}
}
