//! Hardware: receiver, antenna informations
use super::Sv;
use rust_3d::Point3D;

#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "serde")]
use crate::formatter::opt_point3d;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// GNSS receiver description
#[derive(Clone, Debug)]
#[derive(PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Receiver (hardware) model
    pub model: String, 
    /// Receiver (hardware) identification info
    pub sn: String, // serial #
    /// Receiver embedded software info
    pub firmware: String, // firmware #
}

impl Default for Rcvr {
    /// Builds a `default` Receiver
    fn default() -> Rcvr {
        Rcvr {
            model: String::new(),
            sn: String::new(),
            firmware: String::new(),
        }
    }
}

impl std::str::FromStr for Rcvr {
    type Err = std::io::Error;
    fn from_str (line: &str) -> Result<Self, Self::Err> {
        let (id, rem) = line.split_at(20);
        let (make, rem) = rem.split_at(20);
        let (version, _) = rem.split_at(20);
        Ok(Rcvr{
            sn: id.trim().to_string(),
            model: make.trim().to_string(),
            firmware: version.trim().to_string(),
        })
    }
}

/// Antenna description 
#[derive(Debug, Clone, Default)]
#[derive(PartialEq)]
#[cfg_attr(feature = "pyo3", pyclass)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Antenna {
    /// Hardware model / make descriptor
    pub model: String,
    /// Serial number / identification number
    pub sn: String,
    /// Base/reference point coordinates
    #[cfg_attr(feature = "serde", serde(with = "opt_point3d"))]
    pub coords: Option<Point3D>,
    /// Optionnal `h` eccentricity (height component),
    /// referenced to base/reference point, in meter
    pub height: Option<f64>,
    /// Optionnal `eastern` eccentricity (eastern component),
    /// referenced to base/reference point, in meter
    pub eastern: Option<f64>,
    /// Optionnal `northern` eccentricity (northern component),
    /// referenced to base/reference point, in meter
    pub northern: Option<f64>,
}

#[cfg_attr(feature = "pyo3", pymethods)]
impl Antenna {
    /// Sets desired model
    pub fn with_model (&self, m: &str) -> Self {
        let mut s = self.clone();
        s.model = m.to_string();
        s
    }
    /// Sets desired Serial Number
    pub fn with_serial_number (&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.sn = sn.to_string();
        s
    }
    /// Sets reference/base coordinates (3D)
    pub fn with_base_coordinates (&self, x: f64, y: f64, z: f64) -> Self {
        let mut s = self.clone();
        s.coords = Some(Point3D::new(x,y,z));
        s
    }
    /// Sets antenna `h` eccentricity component 
    pub fn with_height(&self, h: f64) -> Self {
        let mut s = self.clone();
        s.height = Some(h);
        s
    }
    /// Sets antenna `eastern` coordinates component
    pub fn with_eastern_component(&self, e: f64) -> Self {
        let mut s = self.clone();
        s.eastern = Some(e);
        s
    }
    /// Sets antenna `northern` coordiantes component
    pub fn with_northern_component(&self, n: f64) -> Self {
        let mut s = self.clone();
        s.northern = Some(n);
        s
    }
}

/// Space vehicule antenna information,
/// only exists in ANTEX records
#[derive(Clone, Debug, Default)]
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SvAntenna {
    /// vehicule this antenna is attached to
    pub sv: Sv,
    /// antenna model description
    pub model: String,
    /// "YYYY-XXXA" year of vehicule launch
    /// XXX sequential launch vehicule
    /// A: alpha numeric sequence number within launch
    pub cospar: Option<String>,
}

impl SvAntenna {
    pub fn with_sv(&self, sv: Sv) -> Self {
        let mut s = self.clone();
        s.sv = sv;
        s
    }
    pub fn with_model(&self, m: &str) -> Self {
        let mut s = self.clone();
        s.model = m.to_string();
        s
    }
    pub fn with_cospar(&self, c: &str) -> Self {
        let mut s = self.clone();
        s.cospar = Some(c.to_string());
        s
    }
}
