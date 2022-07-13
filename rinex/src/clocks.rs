//! clocks.rs   
//! macros and structures for RINEX Clocks files
use crate::sv;
use crate::epoch;
use crate::header;
use thiserror::Error;
use std::str::FromStr;
use std::collections::HashMap;

/// Clocks `RINEX` specific header fields
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Types of observation in this file
    pub codes: Vec<DataType>,
    /// Clock Data analysis production center
    pub agency: Option<Agency>,
    /// Reference station
    pub station: Option<Station>,
    /// Reference clock descriptor
    pub clock_ref: Option<String>,
}

/// Describes a clock station 
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Station {
    /// Station name
    pub name: String,
    /// Station official ID#
    pub id: String,
}

/// Describes a clock analysis center / agency
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct Agency {
    /// IGS AC 3 letter code
    pub code: String,
    /// agency name
    pub name: String,
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
}

#[derive(Error, Clone, Debug)]
pub enum System {
    /// Sv system for AS data
    Sv(sv::Sv),
    /// Stations or Receiver name for other types of data 
    Station(String),
}

impl System {
    /// Unwraps self as a `satellite vehicule`
    pub fn as_sv (&self) -> Option<sv::Sv> {
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
#[derive(Debug, Clone)]
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
pub type Record = HashMap<epoch::Epoch, HashMap<DataType, HashMap<System, Data>>>;

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub fn build_record_entry (header: &header::Header, content: &str) -> 
        Result<(epoch::Epoch, DataType, System, Data), Error> 
{
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let data_type = DataType::from_str(dtype)?;
    let (system_str, rem) = rem.split_at(4);
    let system = match sv::Sv::from_str(system_str) {
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
    line = rem.clone();
    let mut offset = 0;
    let mut n : u8 = 0;
    let mut bias: f64 = 0.0;
    let mut bias_sigma: f64 = 0.0;
    let mut rate: f64 = 0.0;
    let mut rate_sigma: f64 = 0.0;
    let mut accel: f64 = 0.0;
    let mut accel_sigma: f64 = 0.0;
    loop {
        if n == m {
            break
        }
        if n == 2 && m > 2 {
            if let Some(l) = lines.next() {
                offset = 0;
                line = l
            } else {
                break
            }
        }
        n += 1;
        offset += 20;
        let (l, rem) = line.split_at(offset);
        line = rem.clone()
    }
    let data = Data {
        bias,
        bias_sigma: {
            if m > 1 {
                Some(bias_sigma)
            } else {
                None
            }
        },
        rate: {
            if m > 2 {
                Some(rate)
            } else {
                None
            }
        },
        rate_sigma: {
            if m > 3 {
                Some(rate_sigma)
            } else {
                None
            }
        },
        accel: {
            if m > 4 {
                Some(accel)
            } else {
                None
            }
        },
        accel_sigma: {
            if m > 5 {
                Some(accel_sigma)
            } else {
                None
            }
        },
    };
    let epoch = epoch::Epoch {
        flag: epoch::EpochFlag::Ok,
        date,
    };
    Ok((epoch, data_type, system, data))
}
