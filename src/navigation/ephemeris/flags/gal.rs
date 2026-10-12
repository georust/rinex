use crate::{navigation::NavMessageType, ParsingError};
use bitflags::bitflags;

#[cfg(feature = "serde")]
use serde::Serialize;

bitflags! {
    /// [GalHealth] SV ealth indication
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalHealth : u32 {
        const E1B_DVS = 0x00000001;
        const E1B_HS_BIT0 = 0x00000002;
        const E1B_HS_BIT1 = 0x00000004;
        const E5A_DVS = 0x00000008;
        const E5A_HS_BIT0 = 0x00000010;
        const E5A_HS_BIT1 = 0x00000020;
        const E5B_DVS = 0x00000040;
        const E5B_HS_BIT0 = 0x00000080;
        const E5B_HS_BIT1 = 0x00000100;
    }
}

bitflags! {
    /// Galileo data-source word (RINEX 3.04 Table A8 and 4.02 Table A13).
    #[derive(Debug, Default, Copy, Clone)]
    #[derive(PartialEq, PartialOrd)]
    #[cfg_attr(feature = "serde", derive(Serialize))]
    pub struct GalDataSource : u32 {
        /// I/NAV E1-B; may be combined with [GalDataSource::INAV_E5B_I].
        const INAV_E1B = 0x00000001;

        /// F/NAV E5a-I.
        const FNAV_E5A_I = 0x00000002;

        /// I/NAV E5b-I; may be combined with [GalDataSource::INAV_E1B].
        const INAV_E5B_I = 0x00000004;

        /// Reserved for Galileo internal use
        const GAL_RESERVED1 = 0x00000008;

        /// Reserved for Galileo internal use
        const GAL_RESERVED2 = 0x00000010;

        /// af0-af2, Toc and SISA refer to the E1/E5a frequency pair.
        /// Exclusive with [GalDataSource::TEMPORAL_SISA_E1_E5B].
        const TEMPORAL_SISA_E1_E5A= 0x00000100;

        /// af0-af2, Toc and SISA refer to the E1/E5b frequency pair.
        /// Exclusive with [GalDataSource::TEMPORAL_SISA_E1_E5A].
        const TEMPORAL_SISA_E1_E5B= 0x00000200;
    }
}

impl GalDataSource {
    pub(crate) fn from_rinex_value(value: f64) -> Result<Self, ParsingError> {
        if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
            return Err(ParsingError::NavGalileoDataSources);
        }
        // RINEX 3.04 section 8.3.2 / 4.02 section 6.10: nearest integer.
        let rounded = value.round();
        if rounded > u32::MAX as f64 {
            return Err(ParsingError::NavGalileoDataSources);
        }
        Self::from_bits(rounded as u32).ok_or(ParsingError::NavGalileoDataSources)
    }

    pub(crate) fn message_type(self) -> Result<NavMessageType, ParsingError> {
        let clock_refs = Self::TEMPORAL_SISA_E1_E5A | Self::TEMPORAL_SISA_E1_E5B;
        if self.contains(clock_refs) {
            return Err(ParsingError::NavGalileoDataSources);
        }
        let inav = self.intersects(Self::INAV_E1B | Self::INAV_E5B_I);
        let fnav = self.contains(Self::FNAV_E5A_I);
        // Older tables permit merged sources, but they cannot be represented
        // by the current single-message identity. Do not guess a message type.
        match (inav, fnav) {
            (true, false) => Ok(NavMessageType::INAV),
            (false, true) => Ok(NavMessageType::FNAV),
            _ => Err(ParsingError::NavGalileoDataSources),
        }
    }
}
