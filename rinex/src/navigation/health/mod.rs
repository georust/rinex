//! Sv Health specifications
use bitflags::bitflags;

mod gps;
pub use gps::GPSHealth;

mod gal;
pub use gal::GALHealth;

mod qzss;
pub use qzss::QZSSHealth;

/// Sv Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Health {
    /// GPS Sv Health indication
    GPS(GPSHealth),
    /// GAL Sv Health indication
    GAL(GALHealth),
    /// QZSS Sv Health indication
    QZSS(QZSSHealth),
}

impl Default for Health {
    fn default() -> Self {
        Self::GPS(GPSHealth::default())
    }
}

impl From<GPSHealth> for Health {
    fn from(h: GPSHealth) -> Self {
        Self::GPS(h)
    }
}

impl From<GALHealth> for Health {
    fn from(h: GALHealth) -> Self {
        Self::GAL(h)
    }
}

impl From<QZSSHealth> for Health {
    fn from(h: QZSSHealth) -> Self {
        Self::QZSS(h)
    }
}
/*
 * UpperExp formatter, used when generating a file
 */
impl std::fmt::UpperExp for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        todo!("health::upperExp");
    }
}

/// IRNSS orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum IrnssHealth {
    Healthy = 0,
    #[default]
    Unknown = 1,
}

impl std::fmt::UpperExp for IrnssHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0.0_f64.fmt(f),
            Self::Unknown => 1.0_f64.fmt(f),
        }
    }
}

/// SBAS/GEO orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GeoHealth {
    #[default]
    Unknown = 0,
    Reserved = 8,
}

impl std::fmt::UpperExp for GeoHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unknown => 0.fmt(f),
            Self::Reserved => 8.fmt(f),
        }
    }
}

/// GLO orbit health indication
#[derive(Default, Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GloHealth {
    Healthy = 0,
    #[default]
    Unhealthy = 4,
}

impl std::fmt::UpperExp for GloHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0_0_f64.fmt(f),
            Self::Unhealthy => 4.0_f64.fmt(f),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_gps() {
        assert_eq!(Health::default(), Health::Unhealthy);
        assert_eq!(format!("{:E}", Health::default()), "0E0");
    }
    #[test]
    fn test_irnss() {
        assert_eq!(IrnssHealth::default(), IrnssHealth::Unknown);
        assert_eq!(format!("{:E}", IrnssHealth::default()), "1E0");
    }
    #[test]
    fn test_geo_sbas() {
        assert_eq!(GeoHealth::default(), GeoHealth::Unknown);
        assert_eq!(format!("{:E}", Health::default()), "0E0");
    }
    #[test]
    fn test_glo() {
        assert_eq!(GloHealth::default(), GloHealth::Unhealthy);
        assert_eq!(format!("{:E}", GloHealth::default()), "4E0");
    }
}
