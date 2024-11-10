//! GNSS processing context definition.

use std::path::Path;

use sp3::prelude::SP3;

use qc_traits::Merge;

use crate::context::{BlobData, Error, ProductType, QcContext};

impl BlobData {
    /// Returns reference to inner SP3 data.
    pub fn as_sp3(&self) -> Option<&SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }

    /// Returns mutable reference to inner SP3 data.
    pub fn as_mut_sp3(&mut self) -> Option<&mut SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
}

impl QcContext {
    /// Returns reference to inner [ProductType::HighPrecisionOrbit] data
    pub fn sp3(&self) -> Option<&SP3> {
        self.data(ProductType::HighPrecisionOrbit)?.as_sp3()
    }

    /// Returns mutable reference to inner [ProductType::HighPrecisionOrbit] data
    pub fn sp3_mut(&mut self) -> Option<&mut SP3> {
        self.data_mut(ProductType::HighPrecisionOrbit)?.as_mut_sp3()
    }

    /// Returns true if [ProductType::HighPrecisionOrbit] are present in [QcContext]
    pub fn has_sp3(&self) -> bool {
        self.sp3().is_some()
    }

    /// Returns true if [ProductType::HighPrecisionOrbit] also contains temporal information.
    pub fn sp3_has_clock(&self) -> bool {
        if let Some(sp3) = self.sp3() {
            sp3.sv_clock().count() > 0
        } else {
            false
        }
    }

    /// Load one [ProductType::HighPrecisionOrbit] file into [QcContext].
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3(&mut self, path: &Path, sp3: SP3) -> Result<(), Error> {
        let prod_type = ProductType::HighPrecisionOrbit;
        // extend context blob
        if let Some(paths) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == prod_type {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            if let Some(inner) = self.blob.get_mut(&prod_type).and_then(|k| k.as_mut_sp3()) {
                inner.merge_mut(&sp3)?;
                paths.push(path.to_path_buf());
            }
        } else {
            self.blob.insert(prod_type, BlobData::Sp3(sp3));
            self.files.insert(prod_type, vec![path.to_path_buf()]);
        }
        Ok(())
    }
}
