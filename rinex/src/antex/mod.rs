//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod pcv;
pub mod record;
pub mod antenna;
pub mod frequency;

pub use record::{
	Record, Error,
    is_new_epoch,
    parse_epoch,
};
pub use frequency::{Frequency, Pattern};
pub use antenna::{
	Antenna, Calibration, CalibrationMethod,
};

/// ANTEX special RINEX fields
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Antenna Phase Center Variations type 
    pub pcv: pcv::Pcv, 
    /// Types of relative values, default: "AOAD/M_T"
    pub relative_values: String,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_sn: Option<String>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            pcv: pcv::Pcv::default(),
            relative_values: String::new(),
            reference_sn: None,
        }
    }
}
