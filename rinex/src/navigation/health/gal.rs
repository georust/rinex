//! GAL Sv Health specifications
use bitflags::bitflags;

/// GAL Sv Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GALHealth {
    /// GAL Legacy Sv Health indication
    LNAV(LegacyHealth),
    /// GAL INAV Sv Health indication
    INAV(Health),
    /// GAL FNAV Sv Health indication
    FNAV(Health),
}

impl Default for GALHealth {
    fn default() -> Self {
        Self::LNAV(LegacyHealth::default())
    }
}

impl GALHealth {
    /// Unwraps self as [`LegacyHealth`] indicator
    pub(crate) fn lnav(&self) -> Option<&LegacyHealth> {
        match self {
            Self::LNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`Health`] indicator
    pub(crate) fn fnav(&self) -> Option<&Health> {
        match self {
            Self::FNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`Health`] indicator
    pub(crate) fn inav(&self) -> Option<&Health> {
        match self {
            Self::INAV(h) => Some(h),
            _ => None,
        }
    }
}

bitflags! {
    /// GAL Legacy Sv Health indication.
    /// See [Bibliography::RINEX3] for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LegacyHealth: u64 {
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

impl std::fmt::UpperExp for LegacyHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:e}", self.bits() as f32)
    }
}

bitflags! {
    /// GAL FNAV and INAV Health indications.
    /// See [Bibliography::RINEX4] for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct Health: u64 {
        const L1Healthy = 0x01;
        const L2Healthy = 0x02;
        const L5Healthy = 0x04;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_gal() {
        assert_eq!(GalHealth::default(), GalHealth::empty());
    }
}
