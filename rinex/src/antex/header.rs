//! ANTEX specific header

use crate::antex::{
    pcv::PCV,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Type of Phase Center Variation in use
    pub pcv_type: pcv::PCV,
    /// Possible Serial Number of reference antenna used
    /// in this calibration process.
    pub reference_ant_sn: Option<String>,
}

impl HeaderFields {
    /// Set type of Phase Center Variations
    pub fn with_pcv(&self, pcv: Pcv) -> Self {
        let mut s = self.clone();
        s.pcv_type = pcv;
        s
    }
    /// Sets Reference Antenna serial number
    pub fn with_reference_antenna_sn(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.reference_ant_sn = Some(sn.to_string());
        s
    }
}