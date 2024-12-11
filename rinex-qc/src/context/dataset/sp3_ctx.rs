//! SP3 enhanced user data (for PPP)

use crate::{
    context::{
        Error,
        dataset::{meta::MetaData, UserData},
    },
    prelude::{Merge, ProductType, QcContext},
};

use sp3::prelude::SP3;
use std::path::Path;

impl UserData {
    /// Reference to inner [SP3] data unwrapping attempt.
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::SP3(s) => Some(s),
            _ => None,
        }
    }

    /// Mutable reference to inner [SP3] data unwrapping attempt.
    pub fn as_mut_sp3(&mut self) -> Option<&mut SP3> {
        match self {
            Self::SP3(s) => Some(s),
            _ => None,
        }
    }
}

impl DataSet {
    /// Load a single SP3 file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3<P: AsRef<Path>>(&mut self, path: P, sp3: SP3) -> Result<(), Error> {
        let path = path.as_ref();

        let mut meta = MetaData::new(path)?;
        meta.product_id = ProductType::HighPrecisionOrbit;

        // extend context blob
        if let Some(data) = self.sp3_data_mut() {
            data.merge_mut(&sp3)?;
        } else {
            // insert new entry
            self.data.insert(meta, UserData::SP3(sp3));
        }

        Ok(())
    }

    // /// Determines a [UniqueId] for this [SP3] (infaillible).
    // /// This for example, will return unique GNSS receiver identifier.
    // /// It is [RinexType] dependent.
    // fn unique_sp3_id(sp3: &SP3) -> UniqueId {
    //     UniqueId::Agency(sp3.agency.clone())
    // }

    pub fn sp3_data(&self) -> Option<&SP3> {
        let (_, data) = self
            .user_data
            .iter()
            .filter(|(k, _)| k.product_id == ProductType::HighPrecisionOrbit)
            .reduce(|k, _| k)?;

        let sp3 = data.as_sp3()?;
        Some(sp3)
    }

    pub fn sp3_data_mut(&mut self) -> Option<&mut SP3> {
        let (_, data) = self
            .user_data
            .iter_mut()
            .filter(|(k, _)| k.product_id == ProductType::HighPrecisionOrbit)
            .reduce(|k, _| k)?;

        let sp3 = data.as_mut_sp3()?;
        Some(sp3)
    }

    /// Returns true if [ProductType::HighPrecisionOrbit] are present in Self
    pub fn has_sp3(&self) -> bool {
        self.sp3_data().is_some()
    }

    /// Returns true if High Precision Orbits also contains temporal information.
    pub fn sp3_has_clock(&self) -> bool {
        if let Some(sp3) = self.sp3_data() {
            sp3.sv_clock().count() > 0
        } else {
            false
        }
    }
}
