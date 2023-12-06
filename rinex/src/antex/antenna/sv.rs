use thiserror::Error;

mod cospar;

use crate::antex::{Calibration, CalibrationMethod};

#[derive(Error, Debug, Clone)]
pub enum SVAntennaParsingError {
    CosparParsing(#[from] CosparParsingError),
}

pub struct SVAntenna {
    /// Spacecraft
    pub sv: SV
    /// Antenna block type description (IGS)
    pub ant_type: String,
    /// Cospar information
    pub cospar: Cospar,
    /// Calibration info
    pub calibration: Calibration,
}

pub enum ConstellationSpecific {
    GPS(GPSAntenna),
}

pub struct GPSAntenna {
    
}

