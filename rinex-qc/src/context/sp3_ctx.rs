//! SP3 enhanced user data (for PPP)

use crate::{
    context::{Error, InputKey, UniqueId, UserBlobData, UserData},
    prelude::{Merge, ProductType, QcContext},
};

use sp3::prelude::SP3;
use std::path::Path;

impl UserBlobData {
    /// Reference to inner [SP3] data unwrapping attempt.
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }

    /// Mutable reference to inner [SP3] data unwrapping attempt.
    pub fn as_mut_sp3(&mut self) -> Option<&mut SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
}

impl QcContext {
    /// Load a single SP3 file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3(&mut self, path: &Path, sp3: SP3) -> Result<(), Error> {
        let product_type = ProductType::HighPrecisionOrbit;

        let key = InputKey {
            product_type,
            unique_id: Self::unique_sp3_id(&sp3),
        };

        // extend context blob
        if let Some(data) = self.get_unique_sp3_data_mut(&sp3.agency) {
            data.merge_mut(&sp3)?;
        } else {
            // insert new entry
            let user_data = UserData {
                paths: vec![path.to_path_buf()],
                blob_data: UserBlobData::Sp3(sp3),
            };
            self.user_data.insert(key, user_data);
        }

        Ok(())
    }

    /// Determines a [UniqueId] for this [SP3] (infaillible).
    /// This for example, will return unique GNSS receiver identifier.
    /// It is [RinexType] dependent.
    fn unique_sp3_id(sp3: &SP3) -> UniqueId {
        UniqueId::Agency(sp3.agency.clone())
    }

    pub fn sp3_data(&self) -> Option<&SP3> {
        self.get_per_product_user_data(ProductType::HighPrecisionOrbit)?
            .blob_data
            .as_sp3()
    }

    pub fn sp3_data_mut(&mut self) -> Option<&mut SP3> {
        self.get_per_product_user_data_mut(ProductType::HighPrecisionOrbit)?
            .blob_data
            .as_mut_sp3()
    }

    pub fn get_unique_sp3_data(&self, agency: &str) -> Option<&SP3> {
        let key = InputKey {
            product_type: ProductType::HighPrecisionOrbit,
            unique_id: UniqueId::Agency(agency.to_string()),
        };

        self.get_unique_user_data(&key)?.blob_data.as_sp3()
    }

    pub fn get_unique_sp3_data_mut(&mut self, agency: &str) -> Option<&mut SP3> {
        let key = InputKey {
            product_type: ProductType::HighPrecisionOrbit,
            unique_id: UniqueId::Agency(agency.to_string()),
        };

        self.get_unique_user_data_mut(&key)?.blob_data.as_mut_sp3()
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
