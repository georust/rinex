//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod pcv;
pub mod record;
pub mod antenna;
pub mod frequency;

pub use pcv::Pcv;
pub use record::{
	Record, Error,
    is_new_epoch,
    parse_epoch,
};
pub use frequency::{Frequency, Pattern};
pub use antenna::{
	Antenna, 
    Calibration, CalibrationMethod,
};

/// ANTEX special RINEX fields
#[derive(Clone, Debug, Default)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_sn: Option<String>,
    /// Phase Center Variations type 
    pub pcv: pcv::Pcv, 
}

impl HeaderFields {
    /// Sets Phase Center Variations
    pub fn with_pcv(&self, pcv: Pcv) -> Self {
        let mut s = self.clone();
        s.pcv = pcv;
        s
    }
    /// Sets Reference Antenna serial number
    pub fn with_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.reference_sn = Some(sn.to_string());
        s
    }
}
