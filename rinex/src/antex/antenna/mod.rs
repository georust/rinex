use crate::Epoch;
use strum_macros::EnumString;

/* SV antenna support */
mod sv;

/// Known Calibration Methods
#[derive(Default, Clone, Debug, PartialEq, PartialOrd, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum CalibrationMethod {
    #[strum(serialize = "")]
    Unknown,
    #[default]
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
}

/// Antenna description, as contained in ATX records
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Antenna {
    /// Antenna specific field, either a
    /// spacecraft antenna or a receiver antenna
    pub specific: AntennaSpecific,
    /// dazi azimuth increment
    dazi: u16,
    /// zenith grid definition
    pub zenith_grid: Grid,
    /// number of frequencies
    pub nb_frequencies: usize,
    /// start of validity period
    pub valid_from: Epoch,
    /// end of validity period
    pub valid_until: Epoch,
}

impl Antenna {
    /// Returns whether this calibration is valid or not
    pub is_valid(&self, now: Epoch) -> bool {
        now > self.valid_from && now < self.valid_until
    }
}
    
#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum AntennaSpecific {
    /// Attributes of a receiver antenna
    RXAntenna(RXAntenna),
    /// Attributes of a spacecraft antenna
    SVAntenna(sv::SVAntenna),
}

impl AntennaSpecific {
    /* unwrap as SVAntenna, if possible */
    pub(crate) fn as_sv_antenna(&self) -> Option<SVAntenna> {
        match self {
            Self::SVAntenna(ant) => Some(ant),
            _ => None,
        }
    }
    /* unwrap as RXAntenna, if possible */
    pub(crate) fn as_rx_antenna(&self) -> Option<RXAntenna> {
        match self {
            Self::RXAntenna(ant) => Some(ant),
            _ => None,
        }
    }
}


#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RXAntenna {
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SVAntenna {
    pub sv: SV,
    pub cospar_id: String,
}

/// Antenna attributes, as described in ATX records.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Antenna {
    pub ant_type: String,
    pub sn: String,
    /// Calibration informations
    pub calibration: Calibration,
    /// Increment of the azimuth, in degrees
    pub dazi: f64,
    pub zen: (f64, f64),
    pub dzen: f64,
    /// Optionnal SNX, standard IGS/SNX format,
    /// used when referencing this model
    pub sinex_code: Option<String>,
    /// Optionnal validity: start date
    pub valid_from: Option<Epoch>,
    /// Optionnal end of validity
    pub valid_until: Option<Epoch>,
}

impl Default for Antenna {
    fn default() -> Self {
        Self {
            ant_type: String::from("?"),
            sn: String::from("?"),
            calibration: Calibration::default(),
            dazi: 0.0_f64,
            zen: (0.0_f64, 0.0_f64),
            dzen: 0.0_f64,
            sinex_code: None,
            valid_from: None,
            valid_until: None,
        }
    }
}

impl Antenna {
    pub fn with_type(&self, ant_type: &str) -> Self {
        let mut a = self.clone();
        a.ant_type = ant_type.to_string();
        a
    }
    pub fn with_serial_num(&self, sn: &str) -> Self {
        let mut a = self.clone();
        a.sn = sn.to_string();
        a
    }
    pub fn with_calibration(&self, c: Calibration) -> Self {
        let mut a = self.clone();
        a.calibration = c.clone();
        a
    }
    pub fn with_dazi(&self, dazi: f64) -> Self {
        let mut a = self.clone();
        a.dazi = dazi;
        a
    }
    pub fn with_zenith(&self, zen1: f64, zen2: f64, dzen: f64) -> Self {
        let mut a = self.clone();
        a.zen = (zen1, zen2);
        a.dzen = dzen;
        a
    }
    pub fn with_valid_from(&self, e: Epoch) -> Self {
        let mut a = self.clone();
        a.valid_from = Some(e);
        a
    }
    pub fn with_valid_until(&self, e: Epoch) -> Self {
        let mut a = self.clone();
        a.valid_until = Some(e);
        a
    }
    pub fn with_sinex_code(&self, code: &str) -> Self {
        let mut a = self.clone();
        a.sinex_code = Some(code.to_string());
        a
    }
}
