//! GPS Sv Health specifications
use bitflags::bitflags;

/// GPS Sv Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GPSHealth {
    /// GPS Legacy Sv Health indication
    LNAV(LegacyHealth),
    /// GPS CNAV Sv Health indication
    CNAV(CivilianHealth),
    /// GPS CNV2 Sv Health indication
    CNV2(Civilian2Health),
}

impl Default for GPSHealth {
    fn default() -> Self {
        Self::LNAV(LegacyHealth::default())
    }
}

impl GPSHealth {
    /// Unwraps self as [`LegacyHealth`] indicator
    pub(crate) fn lnav(&self) -> Option<&LegacyHealth> {
        match self {
            Self::LNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`CivilianHealth`] indicator
    pub(crate) fn cnav(&self) -> Option<&CivilianHealth> {
        match self {
            Self::CNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`Civilian2Health`] indicator
    pub(crate) fn cnv2(&self) -> Option<&Civilian2Health> {
        match self {
            Self::CNV2(h) => Some(h),
            _ => None,
        }
    }
}

bitflags! {
    /// GPS Legacy Sv Health indication.
    /// See [Bibliography::RINEX3] for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LegacyHealth: u64 {
        const L1Healthy = 0x01;
        const L2Healthy = 0x02;
        const L5Healthy = 0x04;
    }
}

impl std::fmt::UpperExp for LegacyHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:e}", self.bits() as f32)
    }
}

bitflags! {
    /// GPS CNAV Sv Health indication.
    /// See [Bibliography::RINEX4] for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CivilianHealth: u64 {
        const L1Healthy = 0x01;
        const L2Healthy = 0x02;
        const L5Healthy = 0x04;
    }
}

bitflags! {
    /// GPS CNAV-2 Sv Health indication.
    /// See [Bibliography::RINEX4] for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct Civilian2Health: u64 {
        const L1Healthy = 0x01;
        const L2Healthy = 0x02;
        const L5Healthy = 0x04;
    }
}
