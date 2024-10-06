//! IONEX module
use std::collections::HashMap;
use strum_macros::EnumString;

use crate::prelude::{Epoch, SV};

#[cfg(feature = "processing")]
use crate::prelude::TimeScale;

mod header;

pub use header::{
    ReferenceSystem,
    MappingFunction,
    HeaderFields,
};

pub mod grid;
use crate::linspace::Linspace;
pub use grid::Grid;

#[cfg(feature = "processing")]
use qc_traits::processing::{FilterItem, MaskFilter, MaskOperand};

impl std::fmt::Display for MappingFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CosZ => write!(f, "Cos(z)"),
            Self::QFac => write!(f, "Q-factor"),
        }
    }
}
pub(crate) fn is_new_tec_plane(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub(crate) fn is_new_rms_plane(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

/*
 * Don't know what Height maps are actually
 */
// pub(crate) fn is_new_height_map(line: &str) -> bool {
//     line.contains("START OF HEIGHT MAP")
// }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TEC {
    /// Latitude in Radians
    pub latitude_rad: f64,
    /// Longitude in Radians
    pub longitude_rad: f64,
    /// Altitude in Meters
    pub altitude_m: f64,
    /// TEC value
    pub tec: f64,
    /// RMS(TEC)
    pub rms: Option<f64>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn test_mapping_func() {
        let content = "COSZ";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::CosZ);
        let content = "QFAC";
        let func = MappingFunction::from_str(content);
        assert!(func.is_ok());
        assert_eq!(func.unwrap(), MappingFunction::QFac);
        let content = "DONT";
        let func = MappingFunction::from_str(content);
        assert!(func.is_err());
    }
}
