use std::{path::Path, str::FromStr};

use crate::QcCtxError;

use rinex::prelude::ProductionAttributes as RINexProductionAttributes;

/// [MetaData] used specifically in the case of signal "observations"
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObsMetaData {
    /// [MetaData]
    pub meta: MetaData,
    /// Whether this [MetaData] is considered "rover"
    pub is_rover: bool,
}

impl ObsMetaData {
    pub fn from_meta(meta: MetaData) -> Self {
        Self {
            meta,
            is_rover: true,
        }
    }
}

impl std::fmt::Display for ObsMetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.is_rover {
            write!(f, "(ROVER) {}", self.meta)
        } else {
            write!(f, "{}", self.meta)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaData {
    /// File name
    pub name: String,
    /// File extension
    pub extension: String,
    /// Unique ID (if any)
    pub unique_id: Option<String>,
}

impl std::fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(unique_id) = &self.unique_id {
            write!(f, "({})", unique_id)?;
        }
        Ok(())
    }
}

impl MetaData {
    /// Determine basic [MetaData] from provided [Path].
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, QcCtxError> {
        let path = path.as_ref();

        let mut name = path
            .file_stem()
            .ok_or(QcCtxError::FileName)?
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

    /// Convert this [MetaData] to Rover [ObsMetaData]
    pub fn to_rover_obs_meta(&self) -> ObsMetaData {
        ObsMetaData {
            is_rover: true,
            meta: self.clone(),
        }
    }

    /// Convert this [MetaData] to Base [ObsMetaData]
    pub fn to_base_obs_meta(&self) -> ObsMetaData {
        ObsMetaData {
            is_rover: false,
            meta: self.clone(),
        }
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
