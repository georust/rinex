use crate::{
    context::{meta::MetaData, QcContext},
    QcError,
};

use rinex::prelude::{Rinex, RinexType};

impl QcContext {
    /// Load a single [Rinex] into [QcContext].
    /// File revision must be supported, file must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex(&mut self, meta: &mut MetaData, rinex: Rinex) -> Result<(), QcError> {
        // Classification is rinex type dependent
        let rinex_type = rinex.header.rinex_type;
        match rinex_type {
            RinexType::ObservationData => self.load_observation_rinex(meta, rinex),
            RinexType::NavigationData => self.load_navigation_rinex(rinex),
            _ => Err(QcError::NonSupportedFormat),
        }
    }

    /// Converts all internal Compressed RINEx (CRINEx) to RINEx (if any).
    /// This only impacts possibly loaded Observation RINEX files and has
    /// no effect if none are present.
    pub fn crx2rnx_mut(&mut self) {
        for (_, rinex) in &mut self.obs_dataset {
            rinex.crnx2rnx_mut();
        }
    }

    /// Converts all internal readable RINEx to CRINEx (if any).
    /// This only impacts possibly loaded Observation RINEX files and has
    /// no effect if none are present.
    pub fn rnx2crx_mut(&mut self) {
        for (_, rinex) in &mut self.obs_dataset {
            rinex.rnx2crnx_mut();
        }
    }
}
