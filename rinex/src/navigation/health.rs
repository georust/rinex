/// GNSS / GPS orbit health indication
#[derive(Debug, Clone)]
#[derive(FromPrimitive)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Health {
	Unhealthy = 0,
	L1Healthy = 1,
	L2Healthy = 2,
	L1L2Healthy = 3,
	L5Healthy = 4,
	L1L5Healthy = 5,
	L1L2L5Healthy = 7,
}

impl Default for Health {
    fn default() -> Self {
        Self::Unhealthy
    }
}

impl std::fmt::Display for Health {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unhealthy => 0.fmt(f),
            Self::L1Healthy => 1.fmt(f),
            Self::L2Healthy => 2.fmt(f),
            Self::L1L2Healthy => 3.fmt(f),
            Self::L5Healthy => 4.fmt(f),
            Self::L1L5Healthy => 5.fmt(f),
            Self::L1L2L5Healthy => 7.fmt(f),
        }
	}
}
	
/// IRNSS orbit health indication
#[derive(Debug, Clone)]
#[derive(FromPrimitive)]
#[derive(PartialEq, PartialOrd)]
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

impl std::fmt::Display for IrnssHealth {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0.fmt(f),
            Self::Unknown => 1.fmt(f),
        }
    }
}

/// SBAS/GEO orbit health indication
#[derive(Debug, Clone)]
#[derive(FromPrimitive)]
#[derive(PartialEq, PartialOrd)]
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

impl std::fmt::Display for GeoHealth {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Unknown => 0.fmt(f),
            Self::Reserved => 8.fmt(f),
        }
    }
}

/// GLO orbit health indication
#[derive(Debug, Clone)]
#[derive(FromPrimitive)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GloHealth {
	Healthy = 0,
	Unhealthy = 4,
}

impl Default for GloHealth {
	fn default() -> Self {
		Self::Healthy
	}
}

impl std::fmt::Display for GloHealth {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0.fmt(f),
            Self::Unhealthy => 4.fmt(f),
        }
    }
}

/// GAL orbit health indication
#[derive(Debug, Clone)]
#[derive(FromPrimitive)]
#[derive(PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GalHealth {
    Healthy = 0,
    E1bDvs = 0x01,
    E1bHs0 = 0x02,
    E1bHs1 = 0x04,
    E5aDvs = 0x08,
    E5aHs0 = 0x10,
    E5aHs1 = 0x20,
    E5bHs0 = 0x40,
    E5bHs1 = 0x80,
}

impl Default for GalHealth {
	fn default() -> Self {
		Self::Healthy
	}
}

impl std::fmt::Display for GalHealth {
    fn fmt (&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Healthy => 0.fmt(f),
            Self::E1bDvs => 1.fmt(f),
            Self::E1bHs0 => 2.fmt(f),
            Self::E1bHs1 => 4.fmt(f),
            Self::E5aDvs => 8.fmt(f),
            Self::E5aHs0 => 16.fmt(f),
            Self::E5aHs1 => 32.fmt(f),
            Self::E5bHs0 => 64.fmt(f),
            Self::E5bHs1 => 128.fmt(f),
        }
    }
}
