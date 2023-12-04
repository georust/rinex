//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod antenna;
pub mod frequency;
pub mod pcv;
pub mod record;

pub use pcv::Pcv;
pub use record::Record;
pub use antenna::{Antenna, Calibration, CalibrationMethod};
pub use frequency::{Frequency, Pattern};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Type of Phase Center Variation in use
    pub pcv: pcv::Pcv,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_ant_sn: Option<String>,
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
