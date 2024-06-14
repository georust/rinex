mod obs;
// mod nav;
// mod meteo;
// mod clock;
// mod ionex;
use crate::Error;

use obs::Report as ObsReport;

/// RINEX type dependent report
pub enum RINEXReport {
    Obs(OBSReport),
}

impl RINEXReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        match rnx.rinex_type {
            RinexType::ObservationData => Self::Obs(OBSReport::new(rnx)),
            _ => Err(Error::NonSupportedRINEX),
        }
    }
}
