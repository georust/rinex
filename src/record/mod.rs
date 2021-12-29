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
pub enum RecordItemType {
    Float64,
    FixedPoint(String),
    //Epoch,
}

#[derive(Debug)]
struct RecordItem {
    item_type: RecordItemType,
    item_value: String,
}

#[derive(Error, Debug)]
pub enum RecordItemError {
    #[error("failed to parse int value")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("failed to parse float value")]
    ParseFloatError(#[from] std::num::ParseFloatError),
}

impl RecordItem {
    fn from_string (item_type: RecordItemType, content: &str) 
            -> Result<RecordItem, RecordItemError> 
    {
        let item_value = match item_type {
            RecordItemType::Float64 => f64::from_str(&content.replace("D","e"))?.to_string(),
        };
        Ok(RecordItem {
            item_type,
            item_value
        })
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

