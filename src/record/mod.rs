//! record.rs describes a RINEX file record
use thiserror::Error;
use std::str::FromStr;

use crate::constellation;

pub mod navigation;

#[derive(Debug)]
/// `RinexRecord` describes file internal records
pub enum RinexRecord {
    RinexNavRecord(navigation::NavigationRecord),
}

/// `Epoch` describes a timestamp, observation realization
type Epoch = chrono::NaiveDateTime;

/// ̀`Sv` describes a Satellite Vehicule
#[derive(Debug, PartialEq)]
pub struct Sv {
    constellation: constellation::Constellation,
    prn: u8,
}

/// ̀ Sv` parsing / identification related errors
#[derive(Error, Debug)]
pub enum SvError {
    #[error("unknown constellation marker \"{0}\"")]
    UnidentifiedConstellation(char),
    #[error("failed to parse Sv #PRN")]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl Default for Sv {
    /// Builds a default `Sv`
    fn default() -> Sv {
        Sv {
            constellation: constellation::Constellation::default(),
            prn: 0
        }
    }
}

impl Sv {
    /// Creates a new `Sv` Satellite vehicule descriptor
    fn new (constellation: constellation::Constellation, prn: u8) -> Sv {
        Sv {
            constellation,
            prn,
        }
    }
}

impl std::str::FromStr for Sv {
    type Err = SvError;
    /// Builds an `Sv` from string content
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        let mut prn: u8 = 0;
        let mut constellation = constellation::Constellation::default();
        if s.starts_with('G') {
            constellation = constellation::Constellation::GPS;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('E') {
            constellation = constellation::Constellation::Galileo;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('R') {
            constellation = constellation::Constellation::Glonass;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('S') {
            constellation = constellation::Constellation::Sbas;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('J') {
            constellation = constellation::Constellation::QZSS;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else if s.starts_with('C') {
            constellation = constellation::Constellation::Beidou;
            prn = u8::from_str_radix(&s[1..], 10)?;
        } else {
            prn = u8::from_str_radix(&s, 10)?;
        }
        Ok(Sv{constellation, prn})
    }
}

#[derive(Debug)]
/// `RecordItem` describes all known Rinex Records items
pub enum RecordItem {
    Sv(Sv),
    Float64(f64),
    Epoch(Epoch),
    Unsigned(u32),
    Flag(u8),
}

#[derive(Error, Debug)]
pub enum RecordItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
    #[error("failed to parse sv")]
    SvParsingError(#[from] SvError), 
}

impl RecordItem {
    fn from_string (type_descriptor: &str, content: &str) 
            -> Result<RecordItem, RecordItemError> 
    {
        println!("Building \'{}\' from \"{}\"", type_descriptor, content);
        match type_descriptor {
            //TODO
            // normalement pas besoin du replace D->E pour f64
            // introduire un type fixed point (scaled integer)
            //TODO
            // un type binary peut aider pour les mask..
            // u32 doit suffir
            "sv" => Ok(RecordItem::Sv(Sv::from_str(&content)?)),
            "d19.12" => Ok(RecordItem::Float64(f64::from_str(&content.replace("D","e"))?)),
             "epoch" => {
                let items: Vec<&str> = content.split_ascii_whitespace()
                    .collect();
                let (mut y,mon,day,h,min,s): (i32,u32,u32,u32,u32,f64) =
                    (i32::from_str_radix(items[0], 10)?,
                    u32::from_str_radix(items[1], 10)?,
                    u32::from_str_radix(items[2], 10)?,
                    u32::from_str_radix(items[3], 10)?,
                    u32::from_str_radix(items[4], 10)?,
                    f64::from_str(items[5])?);
                if y < 100 {
                    y += 2000 // 2 digit nb case
                }
                Ok(RecordItem::Epoch(
                    chrono::NaiveDate::from_ymd(y,mon,day)
                        .and_hms(h,min,s as u32)))
            },
            "i1" => Ok(RecordItem::Flag(u8::from_str_radix(&content, 10)?)),
            _ => Err(RecordItemError::UnknownTypeDescriptor(type_descriptor.to_string())),
        }
    }
}
