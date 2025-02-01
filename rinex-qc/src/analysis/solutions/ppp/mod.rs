pub mod summary;
use summary::Summary;

use gnss_rtk::prelude::{Epoch, PVTSolution};

#[derive(Default)]
pub struct QcNavPostPPPSolutions {
    summary: Summary,
}

impl QcNavPostPPPSolutions {
    pub fn new_solution(&mut self, t: Epoch, solution: PVTSolution) {
        self.summary.new_solution(t, solution)
    }
}
