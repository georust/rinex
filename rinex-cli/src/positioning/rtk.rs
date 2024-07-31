use gnss_rtk::prelude::{BaseStation as RTKBaseStation, Carrier, Epoch, Observation, SV};

pub struct BaseStation {}

impl RTKBaseStation for BaseStation {
    fn observe(&mut self, t: Epoch, sv: SV, signal: Carrier) -> Option<Observation> {
        None
    }
}
