use std::str::FromStr;
use crate::sv::Sv;
use crate::epoch;
use thiserror::Error;
use crate::version::Version;
use strum_macros::EnumString;
use std::collections::{BTreeMap, HashMap};

#[derive(Error, PartialEq, Eq, Hash, Clone, Debug)]
#[derive(PartialOrd, Ord)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
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
    pub fn as_sv (&self) -> Option<Sv> {
        match self {
            System::Sv(s) => Some(*s),
            _ => None,
        }
    }
    /// Unwraps self as a `station` identification code
    pub fn as_station (&self) -> Option<String> {
        match self {
            System::Station(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl std::fmt::Display for System {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
    ParseEpochError(#[from] epoch::ParseDateError),
    #[error("failed to parse # of data fields")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse data payload")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("failed to identify observable")]
    ParseObservableError(#[from] strum::ParseError),
}

/// Clocks file payload
#[derive(Clone, Debug)]
#[derive(Default)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(EnumString)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
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
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::AR => f.write_str("AR"),
            Self::AS => f.write_str("AS"),
            Self::CR => f.write_str("CR"),
            Self::DR => f.write_str("DR"),
            Self::MS => f.write_str("MS"),
        }
    }
}

/// RINEX record for CLOCKS files,
/// record is sorted by Epoch, by Clock data type and finally by system
pub type Record = BTreeMap<epoch::Epoch, HashMap<DataType, HashMap<System, Data>>>;

pub fn is_new_epoch (line: &str) -> bool {
    // first 2 bytes match a DataType code
    let content = line.split_at(2).0;
    DataType::from_str(content).is_ok()
}

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub fn build_record_entry (version: Version, content: &str) -> 
        Result<(epoch::Epoch, DataType, System, Data), Error> 
{
    let mut lines = content.lines();
    let line = lines.next()
        .unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let data_type = DataType::from_str(dtype.trim())?; // must pass
    
    let mut rem = rem.clone();
    
    let limit = Version {
        major: 3,
        minor: 04,
    };

    let system : System = match version < limit {
        true => { // old fashion
            let (system_str, r) = rem.split_at(5);
            rem = r.clone();
            if let Ok(svnn) = Sv::from_str(system_str.trim()) {
                System::Sv(svnn)
            } else {
                System::Station(system_str.trim().to_string())
            }
        },
        false => { // modern fashion
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
    let offset = 
        4+1 // Y always a 4 digit number, even on RINEX2
       +2+1 // m
       +2+1  // d
       +2+1  // h
       +2+1  // m
        +11; // s
    let (epoch, rem) = rem.split_at(offset);
    let date = epoch::str2date(epoch.trim())?; 

    // n
    let (n, _) = rem.split_at(4);
    let m = u8::from_str_radix(n.trim(), 10)?;

    // data fields
    let mut data = Data::default();
    let items :Vec<&str> = line
        .split_ascii_whitespace()
        .collect();
    data.bias = f64::from_str(items[9].trim())?; // bias must pass
    if m > 1 {
        if let Ok(f) = f64::from_str(items[10].trim()) {
            data.bias_sigma = Some(f)
        }
    }

    if m > 2 {
        if let Some(l) = lines.next() {
            let line = l.clone();
            let items :Vec<&str> = line
                .split_ascii_whitespace()
                .collect();
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
    
    let epoch = epoch::Epoch {
        flag: epoch::EpochFlag::Ok,
        date,
    };
    Ok((epoch, data_type, system, data))
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
        let c = "CR USNO      1995 07 14 20 59 50.000000  2    0.123456789012E+00  -0.123456789012E-01";
        assert_eq!(is_new_epoch(c), true);
        let c = "AS G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(c), true);
        let c = "A  G16  1994 07 14 20 59  0.000000  2   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(c), false);
    }
}
