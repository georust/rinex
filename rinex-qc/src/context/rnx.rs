use crate::context::{meta::MetaData, QcContext};
use rinex::prelude::{Rinex, RinexType};

impl QcContext {
    /// Load a single [Rinex] into [QcContext].
    /// File revision must be supported, file must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex(&mut self, meta: &mut MetaData, rinex: Rinex) {
        // Classification is rinex type dependent
        let rinex_type = rinex.header.rinex_type;
        match rinex_type {
            RinexType::ObservationData => {
                self.load_observation_rinex(meta, rinex);
            },
            RinexType::NavigationData => {
                self.load_navigation_rinex(meta, rinex);
            },
            _ => {
                panic!("not supported!");
            },
        }
    }
}
