mod obs;
mod nav;
mod meteo;
mod clock;
mod ionex;

use obs::Report as ObsReport;

/// RINEX type dependent report
pub enum RINEXReport {
    Obs(OBSReport),
}
