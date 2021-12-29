use thiserror::Error;
use std::str::FromStr;

pub mod navigation;

#[derive(Debug)]
/// `RinexRecord` describes file internal records
pub enum RinexRecord {
    RinexNavRecord(navigation::NavigationRecord),
}

type Epoch = chrono::NaiveDateTime;

#[derive(Debug)]
pub enum RecordItem {
    Float64(f64),
    Epoch(Epoch),
}

#[derive(Error, Debug)]
pub enum RecordItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("unknown type descriptor \"{0}\"")]
    UnknownTypeDescriptor(String),
}

impl RecordItem {
    fn from_string (type_descriptor: &str, content: &str) 
            -> Result<RecordItem, RecordItemError> 
    {
        match type_descriptor {
            "f64" => {
                Ok(RecordItem::Float64(
                    f64::from_str(&content.replace("D","e"))?))
             },
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
            _ => Err(RecordItemError::UnknownTypeDescriptor(type_descriptor.to_string())),
        }
    }
}

// parse from::str pour fixed point:
// remplacer D par "e"

/*impl std:str::FromStr for RinexRecordItem {
    type Err = RinexRecordItemError;
    /// Builds a new `record item` from string content
    fn from_str (s: &str) -> Result<Self, Self::Err> {
        match Self {
            RinexRecordItem::Integer =>
            RinexRecordItem::Integer =>
            RinexRecordItem::Integer =>
        }
    }
}*/

