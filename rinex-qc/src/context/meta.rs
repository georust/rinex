use std::{path::Path, str::FromStr};

use crate::QcError;

use rinex::prelude::ProductionAttributes as RINexProductionAttributes;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaData {
    /// File name
    pub name: String,
    /// File extension
    pub extension: String,
    /// Unique ID (if any)
    pub unique_id: Option<String>,
}

impl MetaData {
    /// Determine basic [MetaData] from provided [Path].
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, QcError> {
        let path = path.as_ref();

        let mut name = path
            .file_stem()
            .ok_or(QcError::FileName)?
            .to_string_lossy()
            .to_string();

        let mut extension = if name.ends_with(".crx") {
            "crx".to_string()
        } else {
            "".to_string()
        };

        // If this Meta is consitent with modern RINEx V3:
        // simply retain only "name" field
        if let Ok(prod) = RINexProductionAttributes::from_str(&name) {
            if let Some(v3_details) = prod.v3_details {
                name = format!("{}{:02}{}", prod.name, v3_details.batch, v3_details.country);
            }
        }

        if let Some(path_extension) = path.extension() {
            let path_extension = path_extension.to_string_lossy();
            if extension.len() > 0 {
                extension.push('.');
            }
            extension.push_str(&path_extension);
        }

        Ok(Self {
            name,
            extension,
            unique_id: None,
        })
    }

    /// Attach a unique identifier to this [MetaData].
    /// The unique identifier will identify the dataset uniquely
    pub fn set_unique_id(&mut self, unique_id: &str) {
        self.unique_id = Some(unique_id.to_string());
    }
}

#[cfg(test)]
mod test {
    use super::MetaData;

    #[test]
    fn test_meta_data() {
        let path = format!(
            "{}/../test_resources/OBS/V2/aopr0010.17o",
            env!("CARGO_MANIFEST_DIR")
        );

        let meta = MetaData::new(&path).unwrap();

        assert_eq!(meta.name, "aopr0010");
        assert_eq!(meta.extension, "17o");

        let path = format!(
            "{}/../test_resources/CRNX/V3/ESBC00DNK_R_20201770000_01D_30S_MO.crx.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let meta = MetaData::new(&path).unwrap();

        assert_eq!(meta.name, "ESBC00DNK");
        assert_eq!(meta.extension, "crx.gz");
    }
}
