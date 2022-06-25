//! clocks.rs   
//! macros and structures for RINEX Clocks files
//use crate::sv;
//use crate::epoch;
//use crate::header;
//use thiserror::error;
use serde_derive::Serialize;

/// Describes a clock analysis center / agency
#[derive(Clone, Serialize, Debug)]
pub struct AnalysisCenter {
    /// IGS AC 3 letter code
    code: String,
    /// agency name
    agency: String,
}

impl AnalysisCenter {
    pub fn new (code: &str, agency: &str) -> AnalysisCenter {
        AnalysisCenter {
            code: code.to_string(),
            agency: agency.to_string(),
        }
    }
}

/*
#[derive(Error, Debug)]
/// Clocks file parsing & identification errors
pub enum Error {
    #[error("unknown data code \"{0}\"")]
    UnknownDataCode(String),
}

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
            System::Station(s) => Some(*s),
            _ => None,
        }
    }
}

/// Clocks file payload
pub struct ClockData {
    pub bias: f64,
    pub bias_sigma: Option<f64>,
    pub rate: Option<f64>,
    pub rate_sigma: Option<f64>,
    pub accel: Option<f64>,
    pub accel_sigma: Option<f64>,
}

/// Types of clock data
pub enum ClockDataType {
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

impl std::str::FromStr for ClockDataType {
    type Err = Error;
    /// Builds a ClockData type from given official code
    fn from_str (code: &str) -> Result<Self, Self::Err> {
        match code {
            "AR" => Ok(ClockDataType::Ar),
            "AS" => Ok(ClockDataType::As),
            "CR" => Ok(ClockDataType::Cr),
            "DR" => Ok(ClockDataType::Dr),
            "MS" => Ok(ClockDataType::Ms),
            _ => Err(Error::UnknownDataCode(code.to_string)),
        }
    }
} 

/// RINEX record for CLOCKS files
pub type Record = HashMap<epoch::Epoch, HashMap<ClockDataType, HashMap<System, ClockData>>>;

/// Builds `RINEX` record entry for `Clocks` data files.   
/// Returns identified `epoch` to sort data efficiently.  
/// Returns 2D data as described in `record` definition
pub fn build_record_entry (header: &Header, content: &str) -> Result<(epoch::Epoch, HashMap<ClocDataType, ClockData>), Error> {
    let mut lines = content.lines();
    let mut line = lines.next()
        .unwrap();
    // Data type code
    let (dtype, rem) = line.split_at(3);
    let dtype = ClockDataType::from_str(dtype)?;
    // System identification
    let system = match dtype {
        ClockDataType::As => {
            // we expect an `sv` identifier for As data
            let (sv, rem) = rem.split_at(3);
            System::Sv(sv::from_str(sv)?)
        },
        _ => {
            // stations identifier is expected for any other data types
            let (syst, rem) = rem.split_at(4);
            let mut system = syst.to_string();
            if rinex.version.major > 2 {
                // Modern RINEX
                // 5 more characheters
                let (syst, rem) = rem.split_at(6); 
                system.push_str(syst)
            }
            System::Station(system.trim_end()) // trick to handle properly 
                // missing extra characters on RINEX 3
        }
    };
    // Epoch
    let offset = 
        4+1, // Y always a 4 digit number, even on RINEX2
       +2+1, // m
       +2+1  // d
       +2+1  // h
       +2+1  // m
        +11; // s
    let (epoch, rem) = rem.split_at(offset);
    let epoch = epoch::from_str(epoch)?;
    // n
    let (n, rem) = rem.split_at(5);
    let m = u8::from_str_radix(n.trim(), 10)?;
    let mut data : Vec<f64> = Vec::with_capacity(6); // max clock data payload size
    line = rem.clone();
    let mut n : u8 = 0;
    loop {
        if n == m {
            break //DONE
        }
        // parsing all clock data payload
        let (data, l) = line.split_at(13); //TODO
        line = l;
    }
    for i in 0..n {
        let (data, rem) = line.split_at(offset);
        let data = f64::from_str(content)?;
        if i == 0 {
            line = lines.next().unwrap();
        }
        offset += 12; // TODO
        if offset >= 84 { // TODO
            line = lines.next().unwrap()
        }
    }
}*/
