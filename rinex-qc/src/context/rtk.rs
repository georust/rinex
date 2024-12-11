use crate::prelude::QcContext;

impl QcContext {
    pub fn remote_observations_iter(&self) {}

    /// True if this [QcContext] allows post processed navigation
    pub fn allows_rtk_navigation(&self) -> bool {
        let has_user_rover = self.user_rover_data.is_some();
        let mut has_sky_context = self.sky_context.has_identified_data();
        if self.cfg.undefined_should_contribute {
            has_sky_context |= self.sky_context.has_unidentified_data();
        }
        has_user_rover && has_sky_context
    }
}
