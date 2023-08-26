//! QZSS Sv Health specifications
use bitflags::bitflags;

/// QZSS Sv Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum QZSSHealth {
    /// QZSS Legacy Sv Health indication.
    /// Refer to [Bibliography::QzssPnt] 5.4.1.
    LNAV(LegacyHealth),
    /// QZSS CNAV Sv Health indication.
    /// Refer to [Bibliography::QzssPnt].
    CNAV(CivilianHealth),
    /// QZSS CNV2 Sv Health indication.
    /// Refer to [Bibliography::QzssPnt].
    CNV2(Civilian2Health),
}

impl Default for QZSSHealth {
    fn default() -> Self {
        Self::LNAV(LegacyHealth::default())
    }
}

impl QZSSHealth {
    /// Unwraps self as [`LegacyHealth`] indicator
    pub(crate) fn lnav(&self) -> Option<&LegacyHealth> {
        match self {
            Self::LNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`Health`] indicator
    pub(crate) fn cnav(&self) -> Option<&CivilianHealth> {
        match self {
            Self::CNAV(h) => Some(h),
            _ => None,
        }
    }
    /// Unwraps self as [`Health`] indicator
    pub(crate) fn cnv2(&self) -> Option<&Civilian2Health> {
        match self {
            Self::CNV2(h) => Some(h),
            _ => None,
        }
    }
}

bitflags! {
    /// QZSS Legacy Sv Health indication.
    /// See [Bibliography::RINEX3] and [Bibliography::QzssPnt] 5.4.1
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct LegacyHealth: u64 {
        const L1CA_B= 0x01 << 22;
        const L1C_A = 0x01 << 21;
        const L2    = 0x01 << 20;
        const L5    = 0x01 << 19;
        const L1C   = 0x01 << 18;
        const L1C_B = 0x01 << 17;

    }
}

impl std::fmt::UpperExp for LegacyHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:e}", self.bits() as f32)
    }
}

bitflags! {
    /// QZSS CNAV Health indications.
    /// See [Bibliography::RINEX4] and [Bibliography::QzssPnt] 5.4.1
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CivilianHealth: u64 {
        const L5 = 0x01 << 54;
        const L2 = 0x01 << 53;
        const L1 = 0x01 << 52;
    }
}

bitflags! {
    /// QZSS CNV2 Health indications.
    /// See [Bibliography::RINEX4] and [Bibliography::QzssPnt] 5.4.1
    /// for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct Civilian2Health: u64 {
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
