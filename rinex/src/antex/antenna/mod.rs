use crate::linspace::Linspace;
use crate::Epoch;
use strum_macros::EnumString;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

mod sv;
pub use sv::{Cospar, SvAntenna, SvAntennaParsingError};

/// Known Calibration Methods
#[derive(Default, Clone, Debug, PartialEq, PartialOrd, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum CalibrationMethod {
    #[strum(serialize = "")]
    #[default]
    Unknown,
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

/// Calibration information
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Calibration {
    /// Calibration method
    pub method: CalibrationMethod,
    /// Agency who performed this calibration
    pub agency: String,
    /// Date of calibration
    pub date: Epoch,
    /// Number of calibrated antennas
    pub number: u16,
}

/// Antenna description, as contained in ATX records
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Antenna {
    /// Antenna specific field, either a
    /// spacecraft antenna or a receiver antenna
    pub specific: AntennaSpecific,
    /// Information on the calibration process.
    pub calibration: Calibration,
    /// Zenith grid definition.
    /// The grid is expressed in zenith angles for RxAntenneas,
    /// or in nadir Angle for SvAntennas.
    pub zenith_grid: Linspace,
    /// Azmiuth increment
    pub azi_inc: f64,
    /// Start of validity period of this information.
    pub valid_from: Epoch,
    /// End of validity period of this information.
    pub valid_until: Epoch,
    /// SINEX code normalization
    pub sinex_code: String,
}

impl Antenna {
    /// Returns whether this calibration is valid or not
    pub fn is_valid(&self, now: Epoch) -> bool {
        now > self.valid_from && now < self.valid_until
    }
    /// Returns the mean phase center position.
    /// If Self is a Receiver Antenna ([`RxAntenna`]),
    /// the returned position is expressed as an offset to the
    /// Antenna Reference Position (ARP).
    /// If Self is a Spacecraft Antenna ([`SvAntenna`]),
    /// the returned position is expressed as an offset to the Spacecraft
    /// Mass Center.
    fn mean_phase_center(&self, reference: (f64, f64, f64)) -> (f64, f64, f64) {
        (0.0_f64, 0.0_f64, 0.0_f64)
    }
    /// Builds an Antenna with given Calibration infos
    pub fn with_calibration(&self, calib: Calibration) -> Self {
        let mut a = self.clone();
        a.calibration = calib.clone();
        a
    }
    /// Builds an Antenna with given Zenith Grid
    pub fn with_zenith_grid(&self, grid: Linspace) -> Self {
        let mut a = self.clone();
        a.zenith_grid = grid.clone();
        a
    }
    /// Builds an Antenna with given Validity period
    pub fn with_validity_period(&self, start: Epoch, end: Epoch) -> Self {
        let mut a = self.clone();
        a.valid_from = start;
        a.valid_until = end;
        a
    }
    /// Builds an Antenna with given Azimuth increment
    pub fn with_dazi(&self, dazi: f64) -> Self {
        let mut a = self.clone();
        a.azi_inc = dazi;
        a
    }
    /// Add custom specificities
    pub fn with_specificities(&self, specs: AntennaSpecific) -> Self {
        let mut a = self.clone();
        a.specific = specs.clone();
        a
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AntennaSpecific {
    /// Attributes of a receiver antenna
    RxAntenna(RxAntenna),
    /// Attributes of a spacecraft antenna
    SvAntenna(sv::SvAntenna),
}

impl Default for AntennaSpecific {
    fn default() -> Self {
        Self::RxAntenna(RxAntenna::default())
    }
}

impl AntennaSpecific {
    /* unwrap as SVAntenna, if possible */
    pub(crate) fn as_sv_antenna(&self) -> Option<&SvAntenna> {
        match self {
            Self::SvAntenna(ant) => Some(ant),
            _ => None,
        }
    }
    /* unwrap as RxAntenna, if possible */
    pub(crate) fn as_rx_antenna(&self) -> Option<&RxAntenna> {
        match self {
            Self::RxAntenna(ant) => Some(ant),
            _ => None,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RxAntenna {
    /// IGS antenna code
    pub igs_type: String,
    /// Antenna serial number
    pub serial_number: Option<String>,
}
