use crate::prelude::QcContext;

impl QcContext {
    pub fn remote_observations_iter(&self) {}

    /// True if this [QcContext] allows post processed RTK (Real Time Kinematics).
    /// [QcContext] has no means to determine the reference points remain static during application:
    /// only the user can define a correct setup.
    pub fn is_rtk_compatible(&self) -> bool {

        if let Some(user_rover) = &self.user_rover_observations {
            if !self.cfg.undefined_should_contribute {
                if !user_rover.is_uniquely_identified() {
                    return false;
                }
            }
        } else {
            false
        }

        let has_user_rover = self.user_rover_observations.is_some();
        let mut has_sky_context = self.sky_context.has_identified_data();
        if self.cfg.undefined_should_contribute {
            has_sky_context |= self.sky_context.has_unidentified_data();
        }
        has_user_rover && has_sky_context
    }
}
