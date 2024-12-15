use crate::context::MetaData;
use anise::prelude::Orbit;
use rinex::prelude::Rinex;
use crate::context::QcContext;


pub enum ObservationUniqueId {
    /// Receiver model
    Receiver(String),
    /// User ID (not meaningul to us)
    UserID(String)
}

/// User (or main) observation [Rinex] data.
/// Also considered as "rover" in RTK scenario.
pub struct UserRoverData {
    /// [Meta] for this [UserRoverData]
    pub meta: MetaData,
    /// [Rinex] data
    pub data: Rinex,
    /// Possible ground position expressed as [Orbit]
    pub ground_position: Option<Orbit>,
    /// Possible unique [UserRoverData] set identifier
    pub unique_id: Option<ObservationUniqueId>,
}

impl UserRoverData {

    /// True if this [UserRoverData] set may be correctly diffenriated
    /// from another of the same kind.
    pub fn is_uniquely_identified(&self) -> bool {
        self.unique_id.is_some()
    }
}

impl QcContext {

    /// (Re)Design internal structre so that [ObservationUniqueId] now defines
    /// the rover (user) device, while all other parts are considered
    /// reference (base) stations. In roaming applications (non static), all reference
    /// base stations are expected to remain static, otherwise your results will not be correct.
    /// When interested in Real Time Kinematics, you should verify the newly designed internal
    /// structure is RTK compatible, with [Self::is_rtk_compatible()].
    pub fn define_rover(&mut self, id: ObservationUniqueId) {


        reference_remote_observations: HashMap<ObservationUniqueId, Rinex>,
        
        self.reference_remote_observations
            .insert()
    }
}