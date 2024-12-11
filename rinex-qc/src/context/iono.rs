use std::collections::HashMap;

use rinex::prelude::Rinex;

use crate::context::{Error, MetaData};

use qc_traits::{Filter, Merge, Preprocessing, Repair, RepairTrait};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct IonoDataIdentifier {
    pub agency: String,
    pub meta: MetaData,
}

#[derive(Default)]
pub struct IonosphereContext {
    identified: HashMap<IonoDataIdentifier, Rinex>,
    unidentified: HashMap<MetaData, Rinex>,
}

impl IonosphereContext {
    /// Loads a new [Rinex] into this [IonosphereContext]
    pub fn load_rinex(&mut self, meta: &MetaData, data: Rinex) -> Result<(), Error> {
        // try to obtain a unique identifier
        let header = &data.header;

        let unique_key = if let Some(agency) = &header.agency {
            Some(IonoDataIdentifier {
                agency: agency.clone(),
                meta: meta.clone(),
            })
        } else {
            None
        };

        if let Some(unique_key) = unique_key {
            self.load_unique_rinex(unique_key, data)?;
        } else {
            self.load_unsorted_rinex(&meta, data)?
        }

        Ok(())
    }

    fn load_unsorted_rinex(&mut self, meta: &MetaData, data: Rinex) -> Result<(), Error> {
        if let Some(inner) = self.unidentified.get_mut(&meta) {
            inner.merge_mut(&data)?;
        } else {
            self.unidentified.insert(meta.clone(), data);
        }
        Ok(())
    }

    fn load_unique_rinex(
        &mut self,
        unique_id: IonoDataIdentifier,
        data: Rinex,
    ) -> Result<(), Error> {
        if let Some(inner) = self.identified.get_mut(&unique_id) {
            inner.merge_mut(&data);
        } else {
            self.identified.insert(unique_id, data);
        }
        Ok(())
    }

    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, data) in self.unidentified.iter_mut() {
            data.filter_mut(filter);
        }
        for (_, data) in self.identified.iter_mut() {
            data.filter_mut(filter);
        }
    }

    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, data) in self.unidentified.iter_mut() {
            data.repair_mut(repair);
        }
        for (_, data) in self.identified.iter_mut() {
            data.repair_mut(repair);
        }
    }

    /// Returns list of Agencies that provided data for this context
    pub fn agencies_iter(&self) -> Box<dyn Iterator<Item = &String> + '_> {
        Box::new(self.identified.keys().map(|k| &k.agency))
    }

    pub fn has_identified_data(&self) -> bool {
        self.identified.len() > 0
    }

    pub fn has_unidentified_data(&self) -> bool {
        self.unidentified.len() > 0
    }

    pub fn empty_data_set(&self) -> bool {
        !self.has_identified_data() && !self.has_unidentified_data()
    }
}
