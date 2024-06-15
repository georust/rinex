mod obs;
// mod nav;
// mod meteo;
// mod clock;
// mod ionex;
use crate::report::Error;

use qc_traits::html::*;
use qc_traits::processing::{FilterItem, MaskOperand};

use obs::Report as ObsReport;

use rinex::prelude::{Observable, Rinex, RinexType};
use std::collections::HashMap;

/// RINEX type dependent report
pub enum RINEXReport {
    Obs(ObsReport),
}

impl RINEXReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        match rnx.header.rinex_type {
            RinexType::ObservationData => Ok(Self::Obs(ObsReport::new(rnx))),
            _ => Err(Error::NonSupportedRINEX),
        }
    }
}
