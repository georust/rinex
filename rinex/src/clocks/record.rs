use crate::{epoch, merge, merge::Merge, prelude::*, split, split::Split, version::Version};
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
    /// Unwraps self as a `satellite vehicle`
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
    EpochParsingError(#[from] epoch::ParsingError),
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
pub struct ClockData {
    /// Clock bias [s]
    pub bias: f64,
    /// Clock bias deviation
    pub bias_dev: Option<f64>,
    /// Clock drift [s/s]
    pub drift: Option<f64>,
    /// Clock drift deviation
    pub drift_dev: Option<f64>,
    /// Clock drift change [s/s^2]
    pub drift_change: Option<f64>,
    /// Clock drift change deviation
    pub drift_change_dev: Option<f64>,
}

/// Clock data observables
#[derive(Debug, PartialEq, Eq, Hash, Clone, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ClockDataType {
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

impl std::fmt::Display for ClockDataType {
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
pub type Record = BTreeMap<Epoch, HashMap<ClockDataType, HashMap<System, ClockData>>>;

pub(crate) fn is_new_epoch(line: &str) -> bool {
    // first 2 bytes match a ClockDataType code
    let content = line.split_at(2).0;
    ClockDataType::from_str(content).is_ok()
}

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub(crate) fn parse_epoch(
    version: Version,
    content: &str,
) -> Result<(Epoch, ClockDataType, System, ClockData), Error> {
    let mut lines = content.lines();
    let line = lines.next().unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let data_type = ClockDataType::from_str(dtype.trim())?; // must pass
    let mut rem = rem.clone();
    let limit = Version { major: 3, minor: 4 };

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
    let (epoch, _) = epoch::parse_utc(epoch.trim())?;

    // nb of data fields
    let (n, _) = rem.split_at(4);
    let n = n.trim().parse::<u8>()?;

    // data fields
    let mut data = ClockData::default();
    let items: Vec<&str> = line.split_ascii_whitespace().collect();
    data.bias = items[9].trim().parse::<f64>()?; // bias must pass
    if n > 1 {
        if let Ok(f) = items[10].trim().parse::<f64>() {
            data.bias_dev = Some(f)
        }
    }

    if n > 2 {
        if let Some(l) = lines.next() {
            let line = l.clone();
            let items: Vec<&str> = line.split_ascii_whitespace().collect();
            for (i, item) in items.iter().enumerate() {
                if let Ok(f) = item.trim().parse::<f64>() {
                    if i == 0 {
                        data.drift = Some(f);
                    } else if i == 1 {
                        data.drift_dev = Some(f);
                    } else if i == 2 {
                        data.drift_change = Some(f);
                    } else if i == 3 {
                        data.drift_change_dev = Some(f);
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
    data: &HashMap<ClockDataType, HashMap<System, ClockData>>,
) -> Result<String, Error> {
    let mut lines = String::with_capacity(128);
    for (dtype, data) in data.iter() {
        for (system, data) in data.iter() {
            lines.push_str(&format!("{} {} {} ", dtype, system, epoch));
            lines.push_str(&format!("{:.13E} ", data.bias));
            if let Some(sigma) = data.bias_dev {
                lines.push_str(&format!("{:.13E} ", sigma));
            }
            if let Some(drift) = data.drift {
                lines.push_str(&format!("{:.13E} ", drift));
            }
            if let Some(sigma) = data.drift_dev {
                lines.push_str(&format!("{:.13E} ", sigma));
            }
            if let Some(drift_change) = data.drift_change {
                lines.push_str(&format!("{:.13E} ", drift_change));
            }
            if let Some(sigma) = data.drift_change_dev {
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

impl Merge for Record {
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
                                if let Some(data) = data.bias_dev {
                                    if ddata.bias_dev.is_none() {
                                        ddata.bias_dev = Some(data);
                                    }
                                }
                                if let Some(data) = data.drift {
                                    if ddata.drift.is_none() {
                                        ddata.drift = Some(data);
                                    }
                                }
                                if let Some(data) = data.drift_dev {
                                    if ddata.drift_dev.is_none() {
                                        ddata.drift_dev = Some(data);
                                    }
                                }
                                if let Some(data) = data.drift_change {
                                    if ddata.drift_change.is_none() {
                                        ddata.drift_change = Some(data);
                                    }
                                }
                                if let Some(data) = data.drift_change_dev {
                                    if ddata.drift_change_dev.is_none() {
                                        ddata.drift_change_dev = Some(data);
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

impl Split for Record {
    fn split(&self, epoch: Epoch) -> Result<(Self, Self), split::Error> {
        let r0 = self
            .iter()
            .flat_map(|(k, v)| {
                if k <= &epoch {
                    Some((*k, v.clone()))
                } else {
                    None
                }
            })
            .collect();
        let r1 = self
            .iter()
            .flat_map(|(k, v)| {
                if k > &epoch {
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
                            !systems.is_empty()
                        });
                        !dtypes.is_empty()
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
            Filter::Smoothing(_) => todo!(),
            Filter::Decimation(_) => todo!(),
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
        unimplemented!("clocks:record:interpolate_mut()");
    }
}
