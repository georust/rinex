use crate::prelude::QcContext;

#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub enum NaviCompatiblity {
    #[default]
    None,
    SPP,
    CPP,
    PPP,
    PPPUltra,
}

impl QcContext {
    /// True if this [QcContext] allows post processed navigation
    pub fn allows_navigation(&self) -> bool {
        let has_user_rover = self.user_rover_data.is_some();
        let mut has_sky_context = self.sky_context.has_identified_data();
        if self.cfg.undefined_should_contribute {
            has_sky_context |= self.sky_context.has_unidentified_data();
        }
        has_user_rover && has_sky_context
    }

    pub fn navi_compatibility(&self) -> NaviCompatiblity {
        if let Some(user_data) = self.user_rover_data {
            if user_data.is_ppp_compatible() {
                NaviCompatiblity::PPP
            } else {
                if user_data.is_cpp_compatible() {
                    NaviCompatiblity::CPP
                } else if user_data.is_spp_compatible() {
                    NaviCompatiblity::SPP
                } else {
                    NaviCompatiblity::None
                }
            }
        } else {
            NaviCompatiblity::None
        }
    }

    #[cfg(not(feature = "sp3"))]
    /// SP3 support is required for 100% PPP compatibility
    pub fn ppp_ultra_compatible(&self) -> bool {
        false
    }

    #[cfg(feature = "sp3")]
    pub fn is_ppp_ultra_compatible(&self) -> bool {
        // TODO: improve
        //      verify clock().ts and obs().ts do match
        //      and have common time frame
        self.clock_data().is_some() && self.sp3_has_clock() && self.is_ppp_compatible()
    }
}
