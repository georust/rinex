//! Antex - special RINEX type specific structures
use crate::channel;
use thiserror::Error;
use std::collections::BTreeMap;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unknown PCV \"{0}\"")]
    UnknownPcv(String),
}

/// Returns true if this line matches 
/// the beginning of a `epoch` for ATX file (special files),
/// this is not really an epoch but rather a group of dataset
/// for this given antenna, there is no sampling data attached to it.
pub fn is_new_epoch (content: &str) -> bool {
    content.contains("START OF ANTENNA")
}

/// ANTEX Record content,
/// is a list of Antenna with Several `Frequency` items in it.
/// ATX record is not `epoch` iterable.
/// All `epochs_()` related methods would fail.
pub type Record = BTreeMap<Antenna, Vec<Frequency>>;

/// Parses entire Antenna block
/// and all inner frequency entries
pub fn build_record_entry (content: &str) -> Result<(Antenna, Vec<Frequency>), Error> {
    let lines = content.lines();
    let mut antenna = Antenna::default();
    let mut frequency = Frequency::default();
    let mut frequencies: Vec<Frequency> = Vec::new();

    for line in lines {
        let (content, marker) = line.split_at(60);
        let content = content.trim();
        let marker = marker.trim();

        if marker.eq("TYPE / SERIAL NO") {
            antenna.with_serial_num()

        } else if marker.contains("METH / BY") {
            antenna.with_method()

        } else if marker.eq("DAZI") {
            antenna.with_dazi()

        } else if marker.eq("ZEN1 / ZEN2 / DZEN" {
            antenna.with_zen()

        } else if marker.eq("VALID FROM") {
            antenna.with_valid_from();

        } else if marker.eq("VALID UNTIL") {
            antenna.with_valid_from();

        } else if marker.eq("SINEX CODE") {
            antenna.with_sinex_code(sinex)

        } else if marker.eq("NORTH / EAST / UP") { 
            let (north, rem) = line.split_at(10);
            let (east, rem) = rem.split_at(10);
            let (up, _) = rem.split_at(10);
            frequency = frequency
                .with_northern(f64::from_str(north)?)
                .with_eastern(f64::from_str(east)?)
                .with_upper(f64::from_str(up)?);
        
        } else if marker.eq("START OF FREQUENCY") {
            frequency = Frequency::default()

        } else if marker.eq("END OF FREQUENCY") {
        }
    }

    Ok((antenna, frequencies))
}

/// ANTEX special RINEX fields
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub struct HeaderFields {
    /// Antenna Phase Center Variations type 
    pub pcv: Pcv, 
    /// Types of relative values, default: "AOAD/M_T"
    pub relative_values: String,
    /// Optionnal reference antenna Serial Number
    /// used to produce this calibration file
    pub reference_sn: Option<String>,
}

impl Default for HeaderFields {
    fn default() -> Self {
        Self {
            pcv: Pcv::default(),
            relative_values: String::new(),
            reference_sn: None,
        }
    }
}

/// Antenna Phase Center Variation types
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "with-serde", derive(Serialize, Deserialize))]
pub enum Pcv {
    /// Given data is aboslute
    Absolute,
    /// Given data is relative
    Relative,
}

impl Default for Pcv {
    fn default() -> Self {
        Self::Absolute
    }
}

impl std::str::FromStr for Pcv {
    type Err = Error;
    fn from_str (content: &str) -> Result<Self, Self::Err> {
        if content.eq("A") {
            Ok(Self::Absolute)
        } else if content.eq("R") {
            Ok(Self::Relative)
        } else {
            Err(Error::UnknownPcv(content.to_string()))
        }
    }
}

/// Describes an Antenna section inside the ATX record
#[derive(Clone, Debug)]
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

impl Antenna {
    pub fn with_serial_num (&self, sn: String) -> Self {
        let mut a = self.clone();
        a.sn = sn.clone();
        a
    }
    pub fn with_sinex_code (&self, c: String) -> Self {
        let mut a = self.clone();
        a.sinex = c.clone();
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

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Non azimuth dependent pattern
    NonAzimuthDependent(Vec<f64>),
    /// Azimuth dependent pattern
    AzimuthDependent(Vec<f64>),
}

/// Describes a "frequency" section of the ATX record
#[derive(Debug, Clone)]
pub struct Frequency {
    /// Channel, example: L1, L2 for GPS, E1, E5 for GAL...
    pub channel: channel::Channel,
    /// TODO
    pub north: f64,
    /// TODO
    pub east: f64,
    /// TODO
    pub up: f64,
    /// Possibly azimuth dependent pattern
    pub pattern: Pattern, 
}

impl Default for Frequency {
    fn default() -> Self {
        Self {
            channel: channel::Channel::default(),
            north: 0.0_f64,
            east: 0.0_f64,
            up: 0.0_f64,
            pattern: Pattern::default(),
        }
    }
}

impl Frequency {
    /// Returns `Frequency` object with updated `Northern` component
    pub fn with_northern (&self, north: f64) -> Self {
        let mut f = self.clone();
        f.north = north;
        f
    }
    pub fn with_eastern (&self, east: f64) -> Self {
        let mut f = self.clone();
        f.east = east;
        f
    }
    pub fn with_upper (&self, up: f64) -> Self {
        let mut f = self.clone();
        f.up = up;
        f
    }
}
