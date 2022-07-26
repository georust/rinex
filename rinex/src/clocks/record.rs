use std::str::FromStr;
use crate::sv::Sv;
use crate::epoch;
use thiserror::Error;
use std::collections::{BTreeMap, HashMap};

#[derive(Error, PartialEq, Eq, Hash, Clone, Debug)]
pub enum System {
    /// Sv system for AS data
    Sv(Sv),
    /// Stations or Receiver name for other types of data 
    Station(String),
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
}

/// Clocks file payload
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Data {
    /// Clock bias
    pub bias: f64,
    pub bias_sigma: Option<f64>,
    pub rate: Option<f64>,
    pub rate_sigma: Option<f64>,
    pub accel: Option<f64>,
    pub accel_sigma: Option<f64>,
}

/// Types of clock data
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// Data analysis results for receiver clocks
    /// derived from a set of network receivers and satellites
    Ar,
    /// Data analysis results for satellites clocks
    /// derived from a set of network receivers and satellites
    As,
    /// Calibration measurements for a single GNSS receiver
    Cr,
    /// Discontinuity measurements for a single GNSS receiver
    Dr,
    /// Monitor measurements for the broadcast sallite clocks
    Ms
}

impl std::str::FromStr for DataType {
    type Err = Error;
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        match code {
            "AR" => Ok(DataType::Ar),
            "AS" => Ok(DataType::As),
            "CR" => Ok(DataType::Cr),
            "DR" => Ok(DataType::Dr),
            "MS" => Ok(DataType::Ms),
            _ => Err(Error::UnknownDataCode(code.to_string())),
        }
    }
} 

/// RINEX record for CLOCKS files,
/// record is sorted by Epoch then by data type and finaly by `system`
pub type Record = BTreeMap<epoch::Epoch, HashMap<System, HashMap<DataType, Data>>>;

pub fn is_new_epoch (line: &str) -> bool {
    // first 3 bytes match a DataType code
    let content = line.split_at(2).0;
    DataType::from_str(content).is_ok()
}

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub fn build_record_entry (content: &str) -> 
        Result<(epoch::Epoch, System, DataType, Data), Error> 
{
    let mut lines = content.lines();
    let line = lines.next()
        .unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let data_type = DataType::from_str(dtype.trim())?;
    let (system_str, rem) = rem.split_at(4);
    let system = match Sv::from_str(system_str) {
        Ok(sv) => System::Sv(sv),
        _ => System::Station(system_str.trim_end().to_string()),
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
    let date = epoch::str2date(epoch)?; 
    // n
    let (n, rem) = rem.split_at(5);
    let m = u8::from_str_radix(n.trim(), 10)?;

    let (content, rem) = rem.split_at(20);
    let bias = f64::from_str(content.trim())?;
    let bias_sigma :Option<f64> = match m > 1 {
        true => {
            let (content, _) = rem.split_at(20);
            Some(f64::from_str(content.trim())?)
        },
        _ => None,
    };
    let rate: Option<f64> = match m > 2 {
        true => {
            let (content, _) = rem.split_at(20);
            Some(f64::from_str(content.trim())?)
        },
        _ => None,
    };
    let rate_sigma :Option<f64> = match m > 3 {
        true => {
            let (content, _) = rem.split_at(20);
            Some(f64::from_str(content.trim())?)
        },
        _ => None,
    };
    let accel: Option<f64> = match m > 4 {
        true => {
            let (content, _) = rem.split_at(20);
            Some(f64::from_str(content.trim())?)
        },
        _ => None,
    };
    let accel_sigma :Option<f64> = match m > 5 {
        true => {
            let (content, _) = rem.split_at(20);
            Some(f64::from_str(content.trim())?)
        },
        _ => None,
    };
    let data = Data {
        bias,
        bias_sigma,
        rate,
        rate_sigma,
        accel,
        accel_sigma,
    };
    let epoch = epoch::Epoch {
        flag: epoch::EpochFlag::Ok,
        date,
    };
    Ok((epoch, system, data_type, data))
}
    

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_is_new_epoch() {
        let l = "AR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(l), true);
        let l = "RA AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(l), false);
        let l = "DR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(l), true);
        let l = "CR AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(l), true);
        let l = "AS AREQ 1994 07 14 20 59  0.000000  6   -0.123456789012E+00 -0.123456789012E+01"; 
        assert_eq!(is_new_epoch(l), true);
    }
}
