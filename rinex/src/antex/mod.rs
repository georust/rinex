//! Antex (ATX) - special RINEX, for antenna caracteristics
pub mod pcv;
pub mod record;
pub mod antenna;
pub mod frequency;

/// ANTEX special RINEX fields
#[derive(Clone, Debug)]
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
