use cggtts::prelude::Track as CggttsTrack;

use crate::{
    context::{meta::MetaData, QcContext},
    navigation::orbit::OrbitSource,
    QcError,
};

use rinex::prelude::obs::SignalObservation;

use gnss_rtk::prelude::{Candidate, Carrier as RTKCarrier, Config, Epoch, Observation, Solver};

/// [NavCggttsSolver] allows you to collect all [CGGTTS] track
/// from a [QcContext], by deploying the complex CGGTTS sky tracking algorithm
/// along the PPP navigation technique.
pub struct NavCggttsSolver<'a> {
    t: Epoch,
    eos: bool,
    pool: Vec<Candidate>,
    rtk_solver: Solver<OrbitSource<'a>>,
    signal_buffer: Vec<SignalObservation>,
    observations: HashMap<RTKCarrier, Observation>,
    signal_iter: Box<dyn Iterator<Item = (Epoch, &'a SignalObservation)> + 'a>,
}

impl<'a> Iterator for NavCggttsSolver<'a> {
    type Item = Option<CggttsTrack>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl QcContext {
    /// Obtain a [NavCggttsSolver] from your [QcContext], ready to resolve solutions.
    pub fn nav_cggtts_solver(
        &self,
        cfg: Config,
        meta: MetaData,
    ) -> Result<NavCggttsSolver, QcError> {
        self.nav_pvt_solver(cfg, &meta, None)
    }
}
