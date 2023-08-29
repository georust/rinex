//! QZSS SV Health specifications
use bitflags::bitflags;

/// QZSS SV Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum QZSSHealth {
    /// QZSS Legacy SV Health indication
    LNAV(LNAVHealth),
    /// QZSS CNAV SV Health indication
    CNAV(CNAVHealth),
    /// QZSS CNV2 SV Health indication
    CNV2(CNV2Health),
}

impl Default for QZSSHealth {
    fn default() -> Self {
        Self::LNAV(LNAVHealth::default())
    }
}

impl QZSSHealth {
    /// Unwraps self as [`LNAVHealth`] indicator
    pub(crate) fn lnav(&self) -> Option<&LNAVHealth> {
        match self {
            Self::LNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`CNAVHealth`] indicator
    pub(crate) fn cnav(&self) -> Option<&CNAVHealth> {
        match self {
            Self::CNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`CNV2Health`] indicator
    pub(crate) fn cnv2(&self) -> Option<&CNV2Health> {
        match self {
            Self::CNV2(h) => Some(h),
            _ => None,
        }
    }
}

bitflags! {
    /// QZSS LNAV SV Health indication.
    /// See [Bibliography::RINEX3] and [Bibliography::QzssPnt] 5.4.1
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LNAVHealth: u64 {
        /// L1 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1    = 0x20;
        /// L1 C/A Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1_CA = 0x10;
        /// L2 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L2    = 0x08;
        /// L5 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L5    = 0x04;
        /// L1C Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1C   = 0x02;
        /// L1 C/B Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1_CB = 0x01;
    }
}

impl std::fmt::UpperExp for LNAVHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:e}", self.bits() as f32)
    }
}

bitflags! {
    /// QZSS CNAV Health indication.
    /// See [Bibliography::RINEX4] and [Bibliography::QzssPnt] 5.4.1 for more information.
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CNAVHealth: u8 {
        /// L1 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1 = 0x01;
        /// L2 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L2 = 0x02;
        /// L5 Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L5 = 0x04;
    }
}

bitflags! {
    /// QZSS CNAV-2 Health indication.
    /// See [Bibliography::RINEX4] and [Bibliography::QzssPnt] 5.4.1 for more information.
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CNV2Health: u8 {
        /// L1C Health
        /// `false` = Healthy
        /// `true` = Unhealthy
        const L1C = 0x01;
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
