use gnss_rtk::prelude::PVTSolution;
use rinex::prelude::{nav::Orbit, Epoch, SV};

#[derive(Default)]
pub struct Summary {
    is_first: bool,
    first_epoch: Epoch,
    last_epoch: Epoch,
    satellites: Vec<SV>,
    initial_rx_orbit: Option<Orbit>,
}

impl Summary {
    /// Latch new [PVTSolution] that has just been resolved
    pub fn new_solution(&mut self, t: Epoch, solution: PVTSolution) {
        if self.is_first {
            self.first_epoch = t;
        } else {
            self.last_epoch = t;
        }
    }
}
