use strum_macros::EnumString;
use crate::antex::frequency::Frequency;

/// Known Calibration Methods
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
#[derive(EnumString)]
pub enum Method {
    #[strum(serialize = "CHAMBER")]
    Chamber,
    #[strum(serialize = "FIELD")]
    Field,
    #[strum(serialize = "ROBOT")]
    Robot,
    /// Copied from other antenna 
    #[strum(serialize = "COPIED")]
    Copied,
    /// Converted from igs_01.pcv or blank
    #[strum(serialize = "CONVERTED")]
    Converted,
}

impl Default for Method {
    fn default() -> Self {
        Self::Chamber
    }
}

/// Calibration information
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Calibration {
    /// Calibration method
    pub method: Method,
    /// Agency who performed the calibration
    pub agency: String,
    /// Date of calibration
    pub date: String,
}

impl Default for Calibration {
    fn default() -> Self {
        Self {
            method: Method::default(),
            agency: String::from("Unknown"),
            date: String::from("Unknown"),
        }
    }
}

/// Describes an Antenna section inside the ATX record
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Antenna {
    pub ant_type: String,
    pub sn: String,
    /// Calibration informations
    pub calibration: Calibration,
    /// Increment of the azimuth, in degrees
    pub dazi: f64,
    pub zen: (f64, f64),
    pub dzen: f64,
    pub valid_from: chrono::NaiveDateTime,
    pub valid_until: chrono::NaiveDateTime,
}

impl Default for Antenna {
    fn default() -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            ant_type: String::from("?"),
            sn: String::from("?"),
            calibration: Calibration::default(),
            dazi: 0.0_f64,
            zen: (0.0_f64, 0.0_f64),
            dzen: 0.0_f64,
            valid_from: now,
            valid_until: now,
        }
    }
}

impl Antenna {
    pub fn with_type (&self, ant_type: &str) -> Self {
        let mut a = self.clone();
        a.ant_type = ant_type.to_string();
        a
    }
    pub fn with_serial_num (&self, sn: &str) -> Self {
        let mut a = self.clone();
        a.sn = sn.to_string();
        a
    }
    pub fn with_calibration (&self, c: Calibration) -> Self {
        let mut a = self.clone();
        a.calibration = c.clone();
        a
    }
    pub fn with_dazi (&self, dazi: f64) -> Self {
        let mut a = self.clone();
        a.dazi = dazi;
        a
    }
    pub fn with_valid_from (&self, v: chrono::NaiveDateTime) -> Self {
        let mut a = self.clone();
        a.valid_from = v.clone();
        a
    }
    pub fn with_valid_until (&self, v: chrono::NaiveDateTime) -> Self {
        let mut a = self.clone();
        a.valid_until = v.clone();
        a
    }
    pub fn with_frequency (&self, f: Frequency) -> Self {
        let mut a = self.clone();
        a
    }
}
