//! GAL SV Health specifications

/// GAL INAV & FNAV SV Health indication.
/// See [Bibliography::RINEX3] for more information.
#[derive(Default, Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct GALHealth {
    /// E1-B Data Validity Status
    /// `false` = Navigation data valid
    /// `true` = Working without guarantee
    e1b_dvs: bool,
    /// E1-B/C Signal Health Status
    /// 0 = Signal OK
    /// 1 = Signal out of service
    /// 2 = Signal will be out of service
    /// 3 = Signal Component currently in Test
    e1b_hs: u8,
    /// E5a Data Validity Status
    /// `false` = Navigation data valid
    /// `true` = Working without guarantee
    e5a_dvs: bool,
    /// E5a Signal Health Status
    /// 0 = Signal OK
    /// 1 = Signal out of service
    /// 2 = Signal will be out of service
    /// 3 = Signal Component currently in Test
    e5a_hs: u8,
    /// E5b Data Validity Status
    /// `false` = Navigation data valid
    /// `true` = Working without guarantee
    e5b_dvs: bool,
    /// E5b Signal Health Status
    /// 0 = Signal OK
    /// 1 = Signal out of service
    /// 2 = Signal will be out of service
    /// 3 = Signal Component currently in Test
    e5b_hs: u8,
}

impl std::fmt::UpperExp for GALHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let value: u32 = u32::from(self);
        write!(f, "{:e}", value as f32)
    }
}

impl From<u32> for GALHealth {
    fn from(value: u32) -> Self {
        Self {
            e1b_dvs: (value & 0b1) != 0,
            e1b_hs: (value & 0b110 >> 1) as u8,
            e5a_dvs: (value & 0b1000) != 0,
            e5a_hs: (value & 0b110000 >> 4) as u8,
            e5b_dvs: (value & 0b1000000) != 0,
            e5b_hs: (value & 0b110000000 >> 7) as u8,
        }
    }
}

impl From<&GALHealth> for u32 {
    fn from(value: &GALHealth) -> Self {
        let mut ret: u32 = 0;

        if value.e1b_dvs {
            ret |= 0b1;
        }

        ret |= (value.e1b_hs as u32) & 0b11 << 1;

        if value.e5a_dvs {
            ret |= 0b1000;
        }

        ret |= (value.e5a_hs as u32) & 0b11 << 4;

        if value.e5b_dvs {
            ret |= 0b1000000;
        }

        ret |= (value.e5b_hs as u32) & 0b11 << 7;

        ret
    }
}
