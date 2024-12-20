use std::path::Path;

use crate::QcError;

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

        let name = path
            .file_stem()
            .ok_or(QcError::FileName)?
            .to_string_lossy()
            .to_string();

        let mut extension = path
            .extension()
            .ok_or(QcError::FileName)?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            name: if let Some(offset) = name.as_str().find('.') {
                name[..offset].to_string()
            } else {
                name.to_string()
            },
            extension: if let Some(offset) = name.as_str().find('.') {
                extension.insert(0, '.');
                extension.insert_str(0, &name[offset + 1..]);
                extension.to_string()
            } else {
                extension.to_string()
            },
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

        assert_eq!(meta.name, "ESBC00DNK_R_20201770000_01D_30S_MO");
        assert_eq!(meta.extension, "crx.gz");
    }
}
