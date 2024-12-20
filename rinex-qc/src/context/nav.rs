use std::collections::HashMap;

use rinex::prelude::{nav::Orbit, Duration, Epoch, SV};

use rinex::prelude::Rinex;

use crate::context::{MetaData, QcContext, QcError};

use qc_traits::{Filter, Merge, Preprocessing, Repair, RepairTrait};

pub enum NavigationUniqueId {
    Agency(String),
}

impl std::fmt::Display for NavigationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agency(ag) => write!(f, "ag:{}", ag),
        }
    }
}

impl std::str::FromStr for NavigationUniqueId {
    type Err = QcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("ag:") {
            Ok(Self::Agency(s[3..].to_string()))
        } else {
            Err(QcError::DataIndexingIssue)
        }
    }
}

impl QcContext {
    pub fn has_navigation_data(&self) -> bool {
        !self.obs_dataset.is_empty()
    }

    /// Loads a new Navigation [Rinex] into this [QcContext]
    pub(crate) fn load_navigation_rinex(
        &mut self,
        meta: &mut MetaData,
        data: Rinex,
    ) -> Result<(), QcError> {
        // Now proceed to stacking
        if let Some(entry) = self.nav_dataset.get_mut(&meta) {
            entry.merge_mut(&data)?;
        } else {
            self.nav_dataset.insert(meta.clone(), data);
        }

        Ok(())
    }
}
