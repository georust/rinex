use crate::context::MetaData;
use anise::prelude::Orbit;
use rinex::prelude::Rinex;

/// User (or main) observation [Rinex] data.
/// Also considered as "rover" in RTK scenario.
pub struct UserRoverData {
    /// [Meta] for this [UserRoverData]
    pub meta: MetaData,
    /// [Rinex] data
    pub data: Rinex,
    /// Possible ground position expressed as [Orbit]
    pub ground_position: Option<Orbit>,
}
