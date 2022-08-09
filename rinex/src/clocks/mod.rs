//! RINEX Clock files parser & analysis 
pub mod record;

/// Clocks `RINEX` specific header fields
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct HeaderFields {
    /// Types of observation in this file
    pub codes: Vec<record::DataType>,
    /// Clock Data analysis production center
    pub agency: Option<Agency>,
    /// Reference station
    pub station: Option<Station>,
    /// Reference clock descriptor
    pub clock_ref: Option<String>,
}

/// Describes a clock station 
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Station {
    /// Station name
    pub name: String,
    /// Station official ID#
    pub id: String,
}

/// Describes a clock analysis center / agency
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize))]
pub struct Agency {
    /// IGS AC 3 letter code
    pub code: String,
    /// agency name
    pub name: String,
}
