use std::collections::HashMap;

use rinex::prelude::Rinex;

use crate::context::{Error, MetaData};

use qc_traits::{Filter, Merge, Preprocessing, Repair, RepairTrait};

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

/// Global data identifier, allows identifing a specific set within
/// multiple [GlobalData]

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct SkyDataIdentifier {
    pub agency: String,
    pub meta: MetaData,
}

enum SkyDataEnum {
    BrdcNavigation(Rinex),
    #[cfg(feature = "sp3")]
    SP3(SP3),
}

#[derive(Default)]
pub struct SkyContext {
    /// [SkyData] pool for which a correct [SkyDataIdentifier] was found
    identified: HashMap<SkyDataIdentifier, SkyDataEnum>,
    /// Unidentified [SkyData]. Will only contribute if [QcConfig] allows it.
    unidentified: HashMap<MetaData, SkyDataEnum>,
}

impl SkyContext {
    /// Loads a new [Rinex] into this [SkyContext]
    pub fn load_rinex(&mut self, meta: &MetaData, data: Rinex) -> Result<(), Error> {
        // try to obtain a unique identifier
        let header = &data.header;

        let unique_key = if let Some(agency) = &header.agency {
            Some(SkyDataIdentifier {
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
            match inner {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.merge_mut(&data)?,
                SkyDataEnum::SP3(_) => {
                    return Err(Error::DataIndexingIssue);
                },
            }
        } else {
            self.unidentified
                .insert(meta.clone(), SkyDataEnum::BrdcNavigation(data));
        }
        Ok(())
    }

    fn load_unique_rinex(
        &mut self,
        unique_id: SkyDataIdentifier,
        data: Rinex,
    ) -> Result<(), Error> {
        if let Some(inner) = self.identified.get_mut(&unique_id) {
            match inner {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.merge_mut(&data)?,
                SkyDataEnum::SP3(_) => {
                    return Err(Error::DataIndexingIssue);
                },
            }
        } else {
            self.identified
                .insert(unique_id, SkyDataEnum::BrdcNavigation(data));
        }
        Ok(())
    }

    /// Loads a new [SP3] into this [SkyContext]
    #[cfg(feature = "sp3")]
    pub fn load_sp3(&mut self, meta: MetaData, data: SP3) -> Result<(), Error> {
        // SP3 are always uniquely identified
        let key = SkyDataIdentifier {
            meta,
            agency: data.agency.to_string(),
        };

        if let Some(inner) = self.identified.get_mut(&key) {
            match inner {
                SkyDataEnum::SP3(sp3) => sp3.merge_mut(&data)?,
                SkyDataEnum::BrdcNavigation(_) => {
                    return Err(Error::DataIndexingIssue);
                },
            }
        }

        Ok(())
    }

    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, data) in self.unidentified.iter_mut() {
            match data {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.filter_mut(filter),
                #[cfg(feature = "sp3")]
                SkyDataEnum::SP3(sp3) => sp3.filter_mut(filter),
            }
        }
        for (_, data) in self.identified.iter_mut() {
            match data {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.filter_mut(filter),
                #[cfg(feature = "sp3")]
                SkyDataEnum::SP3(sp3) => sp3.filter_mut(filter),
            }
        }
    }

    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, data) in self.unidentified.iter_mut() {
            match data {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.repair_mut(repair),
                _ => {},
            }
        }
        for (_, data) in self.identified.iter_mut() {
            match data {
                SkyDataEnum::BrdcNavigation(rinex) => rinex.repair_mut(repair),
                _ => {},
            }
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
