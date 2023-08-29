//! GPS SV Health specifications
use bitflags::bitflags;

/// GPS SV Health indication
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum GPSHealth {
    /// GPS Legacy SV Health indication
    LNAV(LNAVHealth),
    /// GPS CNAV SV Health indication
    CNAV(CNAVHealth),
    /// GPS CNV2 SV Health indication
    CNV2(CNAVHealth),
}

impl Default for GPSHealth {
    fn default() -> Self {
        Self::LNAV(LNAVHealth::default())
    }
}

impl GPSHealth {
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
    /// Unwraps self as [`CNAVHealth`] indicator
    pub(crate) fn cnv2(&self) -> Option<&CNAVHealth> {
        match self {
            Self::CNV2(h) => Some(h),
            _ => None,
        }
    }
}

/// GPS Legacy SV Health indication.
/// Refer to [Bibliography::RINEX3] and [Bibliography::GpsIcd] 20.3.3.3.1.4 for more information.
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct LNAVHealth {
    /// LNAV data health
    /// `false` = all LNAV data are OK
    /// `true` = some or all LNAV data are bad
    data: bool,
    /// Signal component health
    /// 0 = All Signals OK
    /// 1 = All Signals Weak
    /// 2 = All Signals Dead
    /// 3 = All Signals Have No Data Modulation
    /// 4 = L1 P Signal Weak
    /// 5 = L1 P Signal Dead
    /// 6 = L1 P Signal Has No Data Modulation
    /// 7 = L2 P Signal Weak
    /// 8 = L2 P Signal Dead
    /// 9 = L2 P Signal Has No Data Modulation
    /// 10 = L1C Signal Weak
    /// 11 = L1C Signal Dead
    /// 12 = L1C Signal Has No Data Modulation
    /// 13 = L2C Signal Weak
    /// 14 = L2C Signal Dead
    /// 15 = L2C Signal Has No Data Modulation
    /// 16 = L1 & L2 P Signal Weak
    /// 17 = L1 & L2 P Signal Dead
    /// 18 = L1 & L2 P Signal Has No Data Modulation
    /// 19 = L1 & L2C Signal Weak
    /// 20 = L1 & L2C Signal Dead
    /// 21 = L1 & L2C Signal Has No Data Modulation
    /// 22 = L1 Signal Weak
    /// 23 = L1 Signal Dead
    /// 24 = L1 Signal Has No Data Modulation
    /// 25 = L2 Signal Weak
    /// 26 = L2 Signal Dead
    /// 27 = L2 Signal Has No Data Modulation
    /// 28 = SV Is Temporarily Out
    /// 29 = SV Will Be Temporarily Out
    /// 30 = One Or More Signals Are Deformed, However The Relevant URA Parameters Are Valid
    /// 31 = More Than One Combination Would Be Required To Describe Anomalies
    signals: u8,
}

impl std::fmt::UpperExp for LNAVHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value = u32::from(self);
        write!(f, "{:e}", value as f32)
    }
}

impl From<u32> for LNAVHealth {
    fn from(value: u32) -> Self {
        Self {
            data: (value & 0b100000) != 0,
            signals: (value & 0b11111) as u8,
        }
    }
}

impl From<&LNAVHealth> for u32 {
    fn from(value: &LNAVHealth) -> Self {
        let mut ret: u32 = 0;

        if value.data {
            ret |= 0b100000;
        }

        ret |= (value.signals as u32) & 0b11111;

        ret
    }
}

bitflags! {
    /// GPS CNAV & CNAV-2 SV Health indication.
    /// Refer to [Bibliography::RINEX4] and [Bibliography::GpsIcd] 30.3.3.4.4 for more information.
    #[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct CNAVHealth: u8 {
        /// L1 Signal Health
        /// `false` = Some or all codes and data on this carrier are OK
        /// `true` = All codes and data on this carrier are bad or unavailable
        const L1 = 0x01;
        /// L2 Signal Health
        /// `false` = Some or all codes and data on this carrier are OK
        /// `true` = All codes and data on this carrier are bad or unavailable
        const L2 = 0x02;
        /// L5 Signal Health
        /// `false` = Some or all codes and data on this carrier are OK
        /// `true` = All codes and data on this carrier are bad or unavailable
        const L5 = 0x04;
    }
}
