//! Antex - special RINEX type specific structures
use crate::antex::frequency::Frequency;

/// Describes an Antenna section inside the ATX record
#[derive(Clone, Debug)]
#[derive(PartialEq, PartialOrd)]
pub struct Antenna {
    /// TODO
    pub ant_type: String,
    /// TODO
    pub sn: String,
    /// TODO
    pub method: Option<String>,
    /// TODO
    pub agency: Option<String>,
    /// TODO
    pub date: chrono::NaiveDate,
    /// TODO
    pub dazi: f64,
    /// TODO
    pub zen: (f64, f64),
    /// TODO
    pub dzen: f64,
    /// TODO
    pub valid_from: chrono::NaiveDateTime,
    /// TODO
    pub valid_until: chrono::NaiveDateTime,
}

impl Default for Antenna {
    fn default() -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            ant_type: String::from("?"),
            sn: String::from("?"),
            method: None,
            agency: None,
            date: now.date(),
            dazi: 0.0_f64,
            zen: (0.0_f64, 0.0_f64),
            dzen: 0.0_f64,
            valid_from: now,
            valid_until: now,
        }
    }
}

impl Antenna {
    pub fn with_serial_num (&self, sn: String) -> Self {
        let mut a = self.clone();
        a.sn = sn.clone();
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
