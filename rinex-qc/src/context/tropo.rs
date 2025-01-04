use crate::context::{meta::MetaData, QcContext};

impl QcContext {
    // True if Troposophere bias cancellation can be optimized
    // for rover represeted by this [MetaData]
    pub fn allows_troposphere_model_optimization(&self, meta: &MetaData) -> bool {
        if let Some((_x_ecef_m, _y_ecef_m, _z_ecef_m)) = self.rover_rx_position_ecef(meta) {
            // TODO: we should verify that the meteo measurements are regionnal within a certain margin
            false
        } else {
            false
        }
    }
}
