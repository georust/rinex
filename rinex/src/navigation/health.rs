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
