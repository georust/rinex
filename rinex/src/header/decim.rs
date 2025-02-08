use crate::prelude::{Epoch, Header, TimeScale};

use qc_traits::{Decimate, DecimationFilter};

impl Decimate for Header {
    fn decimate(&self, f: &DecimationFilter) -> Self {
        let mut s = self.clone();
        s.decimate_mut(f);
        s
    }
    fn decimate_mut(&mut self, _: &DecimationFilter) {
        self.program = Some(format!(
            "geo-rust v{}",
            Self::format_pkg_version(env!("CARGO_PKG_VERSION"),)
        ));

        if let Ok(now) = Epoch::now() {
            let now_utc = now.to_time_scale(TimeScale::UTC);
            // self.date =
        }
    }
}
