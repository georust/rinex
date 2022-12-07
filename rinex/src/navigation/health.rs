use bitflags::bitflags;

/// GNSS / GPS orbit health indication
#[derive(Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Health {
    Unhealthy = 0,
    L1Healthy = 1,
    L2Healthy = 2,
    L1L2Healthy = 3,
    L5Healthy = 4,
    L1L5Healthy = 5,
    L2L5Healthy = 6,
    L1L2L5Healthy = 7,
}

impl Default for Health {
    fn default() -> Self {
        Self::Unhealthy
    }
}

impl std::fmt::UpperExp for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unhealthy => 0.0_f64.fmt(f),
            Self::L1Healthy => 1.0_f64.fmt(f),
            Self::L2Healthy => 2.0_f64.fmt(f),
            Self::L1L2Healthy => 3.0_f64.fmt(f),
            Self::L5Healthy => 4.0_f64.fmt(f),
            Self::L1L5Healthy => 5.0_f64.fmt(f),
            Self::L2L5Healthy => 6.0_f64.fmt(f),
            Self::L1L2L5Healthy => 7.0_f64.fmt(f),
        }
    }
}

/// IRNSS orbit health indication
#[derive(Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum IrnssHealth {
    Healthy = 0,
    Unknown = 1,
}

impl Default for IrnssHealth {
    fn default() -> Self {
        Self::Unknown
    }
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
#[derive(Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GeoHealth {
    Unknown = 0,
    Reserved = 8,
}

impl Default for GeoHealth {
    fn default() -> Self {
        Self::Unknown
    }
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
#[derive(Debug, Clone, FromPrimitive, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GloHealth {
    Healthy = 0,
    Unhealthy = 4,
}

impl Default for GloHealth {
    fn default() -> Self {
        Self::Unhealthy
    }
}

impl std::fmt::UpperExp for GloHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0_0_f64.fmt(f),
            Self::Unhealthy => 4.0_f64.fmt(f),
        }
    }
}

bitflags! {
    /// GAL orbit health indication
    #[derive(Default)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalHealth: u8 {
        const E1B_DVS = 0x01;
        const E1B_HS0 = 0x02;
        const E1B_HS1 = 0x04;
        const E5A_DVS = 0x08;
        const E5A_HS0 = 0x10;
        const E5A_HS1 = 0x20;
        const E5B_HS0 = 0x40;
        const E5B_HS1 = 0x80;
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
    #[test]
    fn test_gal() {
        assert_eq!(GalHealth::default(), GalHealth::empty());
    }
}
