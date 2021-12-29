pub mod navigation;

#[derive(Debug)]
/// `RinexRecord` describes file internal records
pub enum RinexRecord {
    RinexNavRecord(navigation::NavigationRecord),
}

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
